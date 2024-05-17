use candid::candid_method;
use candid::{CandidType, Deserialize, Principal};
use std::fmt;

use canister_sig_util::signature_map::LABEL_SIG;
use identity_core::common::Url;
use identity_credential::credential::{CredentialBuilder, Subject};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::iter::repeat;

use ic_cdk::api::{caller, set_certified_data, time};
use ic_cdk_macros::{query, update};
use vc_util::issuer_api::{
    CredentialSpec, GetCredentialRequest, IssueCredentialError, IssuedCredentialData,
    PrepareCredentialRequest, PreparedCredentialData, SignedIdAlias,
};

use canister_sig_util::CanisterSigPublicKey;
use ic_certification::{fork_hash, labeled_hash, Hash};
use serde_bytes::ByteBuf;
use vc_util::{
    did_for_principal, get_verified_id_alias_from_jws, vc_jwt_to_jws, vc_signing_input,
    vc_signing_input_hash, AliasTuple,
};

use lazy_static::lazy_static;

// Assuming these are defined in the same or another module that needs to be imported
extern crate asset_util;

use crate::config::{ASSETS, CONFIG, CREDENTIALS, SIGNATURES};
use identity_core::common::Timestamp;

// The expiration of issued verifiable credentials.
const MINUTE_NS: u64 = 60 * 1_000_000_000;
const VC_EXPIRATION_PERIOD_NS: u64 = 15 * MINUTE_NS;

lazy_static! {
    // Seed and public key used for signing the credentials.
    static ref CANISTER_SIG_SEED: Vec<u8> = hash_bytes("some_random_seed").to_vec();
    static ref CANISTER_SIG_PK: CanisterSigPublicKey = CanisterSigPublicKey::new(ic_cdk::id(), CANISTER_SIG_SEED.clone());
}

#[derive(Debug)]
pub enum SupportedCredentialType {
    VerifiedAdult,
}
impl fmt::Display for SupportedCredentialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SupportedCredentialType::VerifiedAdult => write!(f, "VerifiedAdult"),
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub(crate) enum ClaimValue {
    Boolean(bool),
    Date(String),
    Text(String),
    Number(i64),
    Claim(Claim),
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Claim {
    pub(crate) claims: HashMap<String, ClaimValue>,
}

impl From<ClaimValue> for Value {
    fn from(claim_value: ClaimValue) -> Self {
        match claim_value {
            ClaimValue::Boolean(b) => Value::Bool(b),
            ClaimValue::Date(d) => Value::String(d),
            ClaimValue::Text(t) => Value::String(t),
            ClaimValue::Number(n) => Value::Number(n.into()),
            ClaimValue::Claim(nested_claim) => {
                serde_json::to_value(nested_claim).unwrap_or(Value::Null)
            }
        }
    }
}

impl Claim {
    pub(crate) fn into(self) -> Subject {
        let btree_map: BTreeMap<String, Value> = self
            .claims
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        Subject::with_properties(btree_map)
    }
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub(crate) struct StoredCredential {
    pub(crate) id: String,
    pub(crate) type_: Vec<String>,
    pub(crate) context: Vec<String>,
    pub(crate) issuer: String,
    pub(crate) claim: Vec<Claim>,
}
#[derive(CandidType, Deserialize, Debug)]
pub(crate) enum CredentialError {
    NoCredentialFound(String),
    UnauthorizedSubject(String),
}

// Helper functions for constructing the credential that is returned from the canister

/// Build a credentialSubject {
/// id: SubjectId,
/// otherData
///  }

pub(crate) fn build_claims_into_credential_subjects(
    claims: Vec<Claim>,
    subject: String,
) -> Vec<Subject> {
    claims
        .into_iter()
        .zip(repeat(subject))
        .map(|(c, id)| {
            let mut sub = c.into();
            sub.id = Url::parse(id).ok();
            sub
        })
        .collect()
}

pub(crate) fn add_context(
    mut credential: CredentialBuilder,
    context: Vec<String>,
) -> CredentialBuilder {
    for c in context {
        credential = credential.context(Url::parse(c).unwrap());
    }
    credential
}

fn authorize_vc_request(
    alias: &SignedIdAlias,
    expected_vc_subject: &Principal,
    current_time_ns: u128,
) -> Result<AliasTuple, IssueCredentialError> {
    CONFIG.with_borrow(|config| {
        let config = config.get();

        // check if the ID alias is legitimate and was issued by the internet identity canister
        for idp_canister_id in &config.idp_canister_ids {
            if let Ok(alias_tuple) = get_verified_id_alias_from_jws(
                &alias.credential_jws,
                expected_vc_subject,
                idp_canister_id,
                &config.ic_root_key_raw,
                current_time_ns,
            ) {
                return Ok(alias_tuple);
            }
        }
        Err(IssueCredentialError::InvalidIdAlias(
            "id alias could not be verified".to_string(),
        ))
    })
}

#[update]
#[candid_method(update)]
fn add_credentials(principal: Principal, new_credentials: Vec<StoredCredential>) -> Result<String, CredentialError> {
    let caller = ic_cdk::api::caller();

    // Access the configuration and check if the caller is an authorized issuer
    let is_authorized = CONFIG.with(|config_cell| {
        let config = config_cell.borrow();
        let current_config = config.get();
        current_config.authorized_issuers.contains(&caller)
    });

    if !is_authorized {
        return Err(CredentialError::UnauthorizedSubject(format!(
            "Unauthorized issuer: {}",
            caller.to_text()
        )));
    }

    // If authorized, proceed to add credentials
    CREDENTIALS.with_borrow_mut(|credentials| {
        let entry: &mut Vec<StoredCredential> = credentials.entry(principal).or_insert_with(Vec::new);
        entry.extend(new_credentials.clone());
    });

    return Ok(format!(
        "Credentials added successfully: {:?}",
        new_credentials
    ));
}

#[update]
#[candid_method]
fn update_credential(
    principal: Principal,
    credential_id: String,
    updated_credential: StoredCredential,
) -> Result<String, CredentialError> {
    let caller = ic_cdk::api::caller();

    // Access the configuration and check if the caller is an authorized issuer
    let is_authorized = CONFIG.with(|config_cell| {
        let config = config_cell.borrow();
        let current_config = config.get();
        current_config.authorized_issuers.contains(&caller)
    });

    if !is_authorized {
        return Err(CredentialError::UnauthorizedSubject(format!(
            "Unauthorized issuer: {}",
            caller.to_text()
        )));
    }
    // Access the credentials storage and attempt to update the specified credential
    CREDENTIALS.with_borrow_mut(|credentials| {
        if let Some(creds) = credentials.get_mut(&principal) {
            if let Some(pos) = creds.iter().position(|c| c.id == credential_id) {
                creds[pos] = updated_credential.clone();
                return Ok(format!(
                    "Credential updated successfully: {:?}",
                    updated_credential
                ));
            }
        }
        Err(CredentialError::NoCredentialFound(format!(
            "No credential found with ID {} for principal {}",
            credential_id,
            principal.to_text()
        )))
    })
}

#[update]
#[candid_method]
async fn prepare_credential(
    req: PrepareCredentialRequest,
) -> Result<PreparedCredentialData, IssueCredentialError> {
    // here we need to acquire the user principal and use it instead of caller
    let alias_tuple = match authorize_vc_request(&req.signed_id_alias, &caller(), time().into()) {
        Ok(alias_tuple) => alias_tuple,
        Err(err) => return Err(err),
    };

    let credential_jwt = match prepare_credential_jwt(&req.credential_spec, &alias_tuple) {
        Ok(credential) => credential,
        Err(err) => return Result::<PreparedCredentialData, IssueCredentialError>::Err(err),
    };
    let signing_input =
        vc_signing_input(&credential_jwt, &CANISTER_SIG_PK).expect("failed getting signing_input");
    let msg_hash = vc_signing_input_hash(&signing_input);

    SIGNATURES.with(|sigs| {
        let mut sigs = sigs.borrow_mut();
        sigs.add_signature(&CANISTER_SIG_SEED, msg_hash);
    });
    update_root_hash();
    // return a prepared context
    Ok(PreparedCredentialData {
        prepared_context: Some(ByteBuf::from(credential_jwt.as_bytes())),
    })
}

pub(crate) fn update_root_hash() {
    SIGNATURES.with_borrow(|sigs| {
        ASSETS.with_borrow(|assets| {
            let prefixed_root_hash = fork_hash(
                // NB: Labels added in lexicographic order.
                &assets.root_hash(),
                &labeled_hash(LABEL_SIG, &sigs.root_hash()),
            );

            set_certified_data(&prefixed_root_hash[..]);
        })
    })
}

#[query]
#[candid_method(query)]
fn get_credential(req: GetCredentialRequest) -> Result<IssuedCredentialData, IssueCredentialError> {
    if let Err(err) = authorize_vc_request(&req.signed_id_alias, &caller(), time().into()) {
        return Result::<IssuedCredentialData, IssueCredentialError>::Err(err);
    };
    if let Err(err) = verify_credential_spec(&req.credential_spec) {
        return Result::<IssuedCredentialData, IssueCredentialError>::Err(
            IssueCredentialError::UnsupportedCredentialSpec(err),
        );
    }
    let prepared_context = match req.prepared_context {
        Some(context) => context,
        None => {
            return Result::<IssuedCredentialData, IssueCredentialError>::Err(internal_error(
                "missing prepared_context",
            ))
        }
    };
    let credential_jwt: String = match String::from_utf8(prepared_context.into_vec()) {
        Ok(s) => s,
        Err(_) => {
            return Result::<IssuedCredentialData, IssueCredentialError>::Err(internal_error(
                "invalid prepared_context",
            ))
        }
    };
    let signing_input =
        vc_signing_input(&credential_jwt, &CANISTER_SIG_PK).expect("failed getting signing_input");
    let message_hash = vc_signing_input_hash(&signing_input);
    let sig_result = SIGNATURES.with(|sigs| {
        let sig_map = sigs.borrow();
        let certified_assets_root_hash = ASSETS.with_borrow(|assets| assets.root_hash());
        sig_map.get_signature_as_cbor(
            &CANISTER_SIG_SEED,
            message_hash,
            Some(certified_assets_root_hash),
        )
    });
    let sig = match sig_result {
        Ok(sig) => sig,
        Err(e) => {
            return Result::<IssuedCredentialData, IssueCredentialError>::Err(
                IssueCredentialError::SignatureNotFound(format!(
                    "signature not prepared or expired: {}",
                    e
                )),
            );
        }
    };

    let vc_jws =
        vc_jwt_to_jws(&credential_jwt, &CANISTER_SIG_PK, &sig).expect("failed constructing JWS");
    Result::<IssuedCredentialData, IssueCredentialError>::Ok(IssuedCredentialData { vc_jws })
}

fn internal_error(msg: &str) -> IssueCredentialError {
    IssueCredentialError::Internal(String::from(msg))
}

fn prepare_credential_jwt(
    credential_spec: &CredentialSpec,
    alias_tuple: &AliasTuple,
) -> Result<String, IssueCredentialError> {
    let credential_type = match verify_credential_spec(credential_spec) {
        Ok(credential_type) => credential_type,
        Err(err) => {
            return Err(IssueCredentialError::UnsupportedCredentialSpec(err));
        }
    };
    //currently only supports VerifiedAdults spec
    let credential = verify_authorized_principal(credential_type, alias_tuple)?;
    Ok(build_credential(
        alias_tuple.id_alias,
        credential_spec,
        credential,
    ))
}

struct CredentialParams {
    spec: CredentialSpec,
    subject_id: String,
    credential_id_url: String,
    context: Vec<String>,
    issuer_url: String,
    claims: Vec<Claim>,
    expiration_timestamp_s: u32,
}

fn build_credential(
    subject_principal: Principal,
    credential_spec: &CredentialSpec,
    credential: StoredCredential,
) -> String {
    let params = CredentialParams {
        spec: credential_spec.clone(),
        subject_id: did_for_principal(subject_principal),
        credential_id_url: credential.id,
        context: credential.context,
        issuer_url: credential.issuer,
        expiration_timestamp_s: exp_timestamp_s(),
        claims: credential.claim,
    };
    build_credential_jwt(params)
}

// checks if the user has a credential and returns it
fn verify_authorized_principal(
    credential_type: SupportedCredentialType,
    alias_tuple: &AliasTuple,
) -> Result<StoredCredential, IssueCredentialError> {
    // get the credentials of this user
    if let Some(credentials) = CREDENTIALS.with(|credentials| {
        let credentials = credentials.borrow();
        credentials.get(&alias_tuple.id_dapp).cloned()
    }) {
        for c in credentials {
            if c.type_.contains(&credential_type.to_string()) {
                return Ok(c);
            }
        }
    }
    // no (matching) credential found for this user
    println!(
        "*** principal {} it is not authorized for credential type {:?}",
        alias_tuple.id_dapp.to_text(),
        credential_type
    );
    Err(IssueCredentialError::UnauthorizedSubject(format!(
        "unauthorized principal {}",
        alias_tuple.id_dapp.to_text()
    )))
}

pub(crate) fn verify_credential_spec(
    spec: &CredentialSpec,
) -> Result<SupportedCredentialType, String> {
    match spec.credential_type.as_str() {
        "VerifiedAdult" => Ok(SupportedCredentialType::VerifiedAdult),
        other => Err(format!("Credential {} is not supported", other)),
    }
}

/// Builds a verifiable credential with the given parameters and returns the credential as a JWT-string.
fn build_credential_jwt(params: CredentialParams) -> String {
    let subjects = build_claims_into_credential_subjects(params.claims, params.subject_id);
    let expiration_date = Timestamp::from_unix(params.expiration_timestamp_s as i64)
        .expect("internal: failed computing expiration timestamp");

    let mut credential = CredentialBuilder::default()
        .id(Url::parse(params.credential_id_url).unwrap())
        .issuer(Url::parse(params.issuer_url).unwrap())
        .type_("VerifiedCredential".to_string())
        .type_(params.spec.credential_type)
        .subjects(subjects) // add objects to the credentialSubject
        .expiration_date(expiration_date);

    // add all the context
    credential = add_context(credential, params.context);

    let credential = credential.build().unwrap();
    credential.serialize_jwt().unwrap()
}

#[query]
#[candid_method(query)]
fn get_all_credentials(principal: Principal) -> Result<Vec<StoredCredential>, CredentialError> {
    if let Some(c) = CREDENTIALS.with_borrow(|credentials| credentials.get(&principal).cloned()) {
        Ok(c)
    } else {
        Err(CredentialError::NoCredentialFound(format!(
            "No credentials found for principal {}",
            principal.to_text()
        )))
    }
}

fn hash_bytes(value: impl AsRef<[u8]>) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(value.as_ref());
    hasher.finalize().into()
}

fn exp_timestamp_s() -> u32 {
    ((time() + VC_EXPIRATION_PERIOD_NS) / 1_000_000_000) as u32
}
