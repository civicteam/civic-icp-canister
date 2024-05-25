//! Handles adding, updating, and retrieving verifiable credentials within the Civic Canister.
//!
//! This module provides functionality to manage and manipulate verifiable credentials,
//! including issuing, updating, and retrieving credentials. It also handles authorization
//! and verification processes related to credential operations.
use candid::{candid_method, CandidType, Decode, Deserialize, Encode, Principal};
use canister_sig_util::signature_map::LABEL_SIG;
use canister_sig_util::CanisterSigPublicKey;
use ic_cdk::api::{caller, set_certified_data, time};
use ic_cdk_macros::{query, update};
use ic_certification::{fork_hash, labeled_hash, Hash};
use ic_stable_structures::storable::{Bound, Storable};
use identity_core::common::Timestamp;
use identity_core::common::Url;
use identity_credential::credential::{CredentialBuilder, Subject};
use lazy_static::lazy_static;
use serde::Serialize;
use serde_bytes::ByteBuf;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::iter::repeat;
use vc_util::issuer_api::{
    CredentialSpec, GetCredentialRequest, IssueCredentialError, IssuedCredentialData,
    PrepareCredentialRequest, PreparedCredentialData, SignedIdAlias,
};
use vc_util::{
    did_for_principal, get_verified_id_alias_from_jws, vc_jwt_to_jws, vc_signing_input,
    vc_signing_input_hash, AliasTuple,
};

extern crate asset_util;

use crate::config::{ASSETS, CONFIG, CREDENTIALS, MSG_HASHES, SIGNATURES, LOOKUP_TABLE};

// The expiration of issued verifiable credentials.
const MINUTE_NS: u64 = 60 * 1_000_000_000;
const VC_EXPIRATION_PERIOD_NS: u64 = 15 * MINUTE_NS;

lazy_static! {
    /// Seed and public key used for signing the credentials.
    pub(crate) static ref CANISTER_SIG_SEED: Vec<u8> = hash_bytes("a_random_seed").to_vec();
    static ref CANISTER_SIG_PK: CanisterSigPublicKey = CanisterSigPublicKey::new(ic_cdk::id(), CANISTER_SIG_SEED.clone());
}

/// Supported types of credentials that can be issued by this canister.
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

/// Represents different types of claim values that can be part of a credential.
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum ClaimValue {
    Boolean(bool),
    Date(String),
    Text(String),
    Number(i64),
    Claim(Claim),
}

/// Represents a collection of claims.
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct Claim {
    pub claims: HashMap<String, ClaimValue>,
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

/// Converts a `Claim` into a `Subject` that represents a credential subject containing the given claims (but no subject ID yet)
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

/// Represents a full credential that includes the issuer and context url in full. This is the type that will be passed to the canister
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct FullCredential {
    pub id: String,
    pub type_: Vec<String>,
    pub issuer: String,
    pub context: Vec<String>,
    pub claim: Vec<Claim>,
}

/// Represents a stored credential within the canister in the 'compressed' form with the url fields resolved to the url id
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
struct StoredCredential {
    id: String,
    type_: Vec<String>,
    context_issuer_id: u16,
    claim: Vec<Claim>,
}

/// Convert from a single full credential to a single stored credential
impl From<FullCredential> for StoredCredential {
    fn from(full_credential: FullCredential) -> Self {
        // Get the corresponding key for these values or insert a new entry into the LookupTable
        let url_id = LOOKUP_TABLE.with_borrow_mut(|map| {
            map.get_or_insert(full_credential.issuer, full_credential.context)
        });
        StoredCredential {
            id: full_credential.id,
            type_: full_credential.type_,
            context_issuer_id: url_id,
            claim: full_credential.claim,
        }
    }
}

/// Define a wrapper type around a list of credentials so that we can store it inside Stable Storage as well as implement to and from conversion to a list of full credentials
#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct CredentialList(Vec<StoredCredential>);

/// Implement the trait needed to use CredentialList inside a StableBTreeMap
impl Storable for CredentialList {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&self.0).expect("Failed to encode StoredCredential"))
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        CredentialList(
            Decode!(&bytes, Vec<StoredCredential>).expect("Failed to decode StoredCredential"),
        )
    }

    // This measures the size of the object in bytes
    const BOUND: Bound = Bound::Unbounded;
}

impl From<CredentialList> for Vec<StoredCredential> {
    fn from(credentials: CredentialList) -> Self {
        credentials.0
    }
}

/// Convert from a list of stored credentials to a list of full credentials
impl From<CredentialList> for Vec<FullCredential> {
    fn from(credentials: CredentialList) -> Vec<FullCredential> {
        let mut new_full_credentials: Vec<FullCredential> = Vec::new();
        LOOKUP_TABLE.with(|map| {
            for c in credentials.0 {
                let (issuer, context) = map
                    .borrow()
                    .get(c.context_issuer_id)
                    .unwrap()
                    .to_owned();
                let full_credential = FullCredential {
                    id: c.id,
                    type_: c.type_,
                    issuer,
                    context,
                    claim: c.claim,
                };
                new_full_credentials.push(full_credential);
            }
            new_full_credentials
        })
    }
}

/// Convert from a list of full credentials to a list of stored credentials
impl From<Vec<FullCredential>> for CredentialList {
    fn from(full_credentials: Vec<FullCredential>) -> Self {
        let mut new_stored_credentials: Vec<StoredCredential> = Vec::new();
        // For each full credential get or insert the id for the issuer and context fields, converting it into a StoredCredential type
        for c in full_credentials {
            new_stored_credentials.push(<StoredCredential>::from(c));
        }
        CredentialList(new_stored_credentials)
    }
}

/// Enumerates potential errors that can occur during credential operations.
#[derive(CandidType, Deserialize, Debug)]
pub enum CredentialError {
    NoCredentialFound(String),
    UnauthorizedSubject(String),
}

fn is_authorized_issuer(caller: Principal) -> bool {
    CONFIG.with(|config_cell| {
        let config = config_cell.borrow();
        let current_config = config.get();
        current_config.authorized_issuers.contains(&caller)
    })
}

/// Adds new credentials to the canister for a given principal.
#[update]
#[candid_method]
async fn add_credentials(
    principal: Principal,
    full_credentials: Vec<FullCredential>,
) -> Result<String, CredentialError> {
    // Check if the caller is the authorized principal
    if !is_authorized_issuer(caller()) {
        return Err(CredentialError::UnauthorizedSubject(
            "Unauthorized: You do not have permission to update credentials.".to_string(),
        ));
    }
    // First get the it in compressed form of StoredCredential
    let new_stored_credentials: CredentialList = CredentialList::from(full_credentials.clone());
    // Access the credentials storage and attempt to add the new credentials
    CREDENTIALS.with(|c| {
        // Get a mutable reference to the stable map
        let mut credentials = c.borrow_mut();
        // Check if there is already credentials stored under this principal
        if credentials.contains_key(&principal) {
            let mut existing_credentials: Vec<StoredCredential> =
                credentials.get(&principal).unwrap().into();

            // If yes, add or replace with the new credential
            for new_c in new_stored_credentials.clone().0 {
                if let Some(pos) = existing_credentials
                    .iter()
                    .position(|existing_c| existing_c.id == new_c.id)
                {
                    // Replace existing credential
                    existing_credentials[pos] = new_c;
                } else {
                    // Insert new credential
                    existing_credentials.push(new_c);
                }
            }
            credentials.insert(principal, CredentialList(existing_credentials));
        } else {
            // Else insert the new entry
            credentials.insert(principal, new_stored_credentials);
        }
    });

    let credential_info = format!("Added credentials: \n{:?}", full_credentials);
    Ok(credential_info)
}

#[update]
#[candid_method]
async fn remove_credential(
    principal: Principal,
    credential_id: String,
) -> Result<String, CredentialError> {
    let caller = caller();

    // Check if the caller is an authorized issuer
    if !is_authorized_issuer(caller) {
        return Err(CredentialError::UnauthorizedSubject(
            "Unauthorized: You do not have permission to remove credentials.".to_string(),
        ));
    }

    // Retrieve the full credentials to check the issuer
    let existing_full_credentials = get_all_credentials(principal.clone())?;
    let _ = existing_full_credentials
        .iter()
        .find(|cred| cred.id == credential_id)
        .ok_or_else(|| CredentialError::NoCredentialFound(format!(
            "No credential found with ID {} for principal {}",
            credential_id,
            principal.to_text()
        )))?;

    // Access the credentials storage and attempt to remove the credential
    let result = CREDENTIALS.with(|c| {
        let mut credentials = c.borrow_mut();

        if let Some(existing_credentials) = credentials.get(&principal) {
            let mut existing_credentials_vec: Vec<StoredCredential> = existing_credentials.into();

            if let Some(pos) = existing_credentials_vec
                .iter()
                .position(|cred| cred.id == credential_id)
            {
                existing_credentials_vec.remove(pos);
                credentials.insert(principal, CredentialList(existing_credentials_vec));
                Ok("Credential removed successfully".to_string())
            } else {
                Err(CredentialError::NoCredentialFound(
                    "Credential not found.".to_string(),
                ))
            }
        } else {
            Err(CredentialError::NoCredentialFound(
                "No credentials found for this principal.".to_string(),
            ))
        }
    });

    result
}

/// Updates an existing credential for a given principal.
#[update]
#[candid_method]
async fn update_credential(
    principal: Principal,
    credential_id: String,
    updated_full_credential: FullCredential,
) -> Result<String, CredentialError> {
    let caller = caller();

    // Check if the caller is an authorized issuer
    if !is_authorized_issuer(caller) {
        return Err(CredentialError::UnauthorizedSubject(
            "Unauthorized: You do not have permission to update credentials.".to_string(),
        ));
    }

    // Retrieve the full credentials to check the issuer
    let existing_full_credentials = get_all_credentials(principal.clone())?;
    let _ = existing_full_credentials
        .iter()
        .find(|cred| cred.id == credential_id)
        .ok_or_else(|| CredentialError::NoCredentialFound(format!(
            "No credential found with ID {} for principal {}",
            credential_id,
            principal.to_text()
        )))?;

    // Convert the updated full credential to stored credential
    let updated_stored_credential = StoredCredential::from(updated_full_credential.clone());

    // Access the credentials storage and attempt to update the specified credential
    let result = CREDENTIALS.with(|c| {
        let mut creds = c.borrow_mut();
        if let Some(credentials) = creds.get(&principal) {
            let mut credentials: Vec<StoredCredential> = credentials.into();
            // Iterate through the credentials and find the one with the given id
            if let Some(pos) = credentials.iter().position(|c| c.id == credential_id) {
                // Update the credential with the new data
                credentials[pos] = updated_credential.clone().into();
                // Update the principal with the new list of credentials
                creds.insert(principal, CredentialList(credentials));
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
    });
    result
}

/// Retrieves all credentials for a given principal.
#[query]
#[candid_method(query)]
fn get_all_credentials(principal: Principal) -> Result<Vec<FullCredential>, CredentialError> {
    if let Some(c) = CREDENTIALS.with(|c| c.borrow().get(&principal)) {
        Ok(c.into())
    } else {
        Err(CredentialError::NoCredentialFound(format!(
            "No credentials found for the principal {}",
            principal.to_text()
        )))
    }
}

/// Request to prepare a VC for issuance.
#[update]
#[candid_method]
async fn prepare_credential(
    req: PrepareCredentialRequest,
) -> Result<PreparedCredentialData, IssueCredentialError> {
    // Check if the ID alias of a VC request is valid and matches the expected VC subject.
    let alias_tuple = match authorize_vc_request(&req.signed_id_alias, &caller(), time().into()) {
        Ok(alias_tuple) => alias_tuple,
        Err(err) => return Err(err),
    };

    // Construct the JWT of the VC to be issued.
    let credential_jwt = match prepare_credential_jwt(&req.credential_spec, &alias_tuple) {
        Ok(credential) => credential,
        Err(err) => return Result::<PreparedCredentialData, IssueCredentialError>::Err(err),
    };
    // And sign the JWT
    let signing_input =
        vc_signing_input(&credential_jwt, &CANISTER_SIG_PK).expect("Failed getting signing_input.");
    let msg_hash = vc_signing_input_hash(&signing_input);

    // Add the signed JWT to the signature storage
    SIGNATURES.with(|sigs| {
        let mut sigs = sigs.borrow_mut();
        sigs.add_signature(&CANISTER_SIG_SEED, msg_hash);
        // Add the msg hash to the stable storage to restore the signatures when the canister is upgraded
        MSG_HASHES.with(|hashes| {
            let _ = hashes.borrow_mut().push(&msg_hash);
        });
    });
    update_root_hash();
    // Return a prepared context that includes the signed JWT
    Ok(PreparedCredentialData {
        prepared_context: Some(ByteBuf::from(credential_jwt.as_bytes())),
    })
}

/// Obtain a VC from the canister after it was prepared.
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
    // Check if the prepared context is present in the request. This context should contain the JWT of the VC, get it as a string
    let prepared_context = match req.prepared_context {
        Some(context) => context,
        None => {
            return Result::<IssuedCredentialData, IssueCredentialError>::Err(internal_error(
                "Missing prepared_context",
            ))
        }
    };
    let credential_jwt: String = match String::from_utf8(prepared_context.into_vec()) {
        Ok(s) => s,
        Err(_) => {
            return Result::<IssuedCredentialData, IssueCredentialError>::Err(internal_error(
                "Invalid prepared_context",
            ))
        }
    };

    // Sign the JWT
    let signing_input =
        vc_signing_input(&credential_jwt, &CANISTER_SIG_PK).expect("failed getting signing_input");
    let message_hash = vc_signing_input_hash(&signing_input);
    // Match it to the signature from the signature storage.
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
            // If the signature is not found or has expired, return an error.
            return Result::<IssuedCredentialData, IssueCredentialError>::Err(
                IssueCredentialError::SignatureNotFound(format!(
                    "Signature not prepared or expired: {}",
                    e
                )),
            );
        }
    };

    let vc_jws =
        vc_jwt_to_jws(&credential_jwt, &CANISTER_SIG_PK, &sig).expect("failed constructing JWS");
    Result::<IssuedCredentialData, IssueCredentialError>::Ok(IssuedCredentialData { vc_jws })
}

/// Check if the ID alias of a VC request is valid and matches the expected VC subject.
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
            "Id alias could not be verified".to_string(),
        ))
    })
}

/// Check if the given user has a credential of the type and return it.
fn verify_authorized_principal(
    credential_type: SupportedCredentialType,
    alias_tuple: &AliasTuple,
) -> Result<StoredCredential, IssueCredentialError> {
    // Get the credentials of this user
    if let Some(credentials) = CREDENTIALS.with(|c| c.borrow().get(&alias_tuple.id_dapp)) {
        // Check if the user has a credential of the type and return it
        let v: Vec<StoredCredential> = credentials.into();
        for c in v {
            if c.type_.contains(&credential_type.to_string()) {
                return Ok(c);
            }
        }
    }
    // No (matching) credential found for this user
    println!(
        "*** Principal {} it is not authorized for credential type {:?}",
        alias_tuple.id_dapp.to_text(),
        credential_type
    );
    Err(IssueCredentialError::UnauthorizedSubject(format!(
        "Unauthorized principal {}",
        alias_tuple.id_dapp.to_text()
    )))
}

/// Verifies if the credential spec is supported and returns the corresponding credential type.
pub(crate) fn verify_credential_spec(
    spec: &CredentialSpec,
) -> Result<SupportedCredentialType, String> {
    match spec.credential_type.as_str() {
        "VerifiedAdult" => Ok(SupportedCredentialType::VerifiedAdult),
        other => Err(format!("Credential {} is not supported", other)),
    }
}

fn internal_error(msg: &str) -> IssueCredentialError {
    IssueCredentialError::Internal(String::from(msg))
}

fn hash_bytes(value: impl AsRef<[u8]>) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(value.as_ref());
    hasher.finalize().into()
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
    // Currently only supports VerifiedAdults spec
    let credential = verify_authorized_principal(credential_type, alias_tuple)?;
    Ok(build_credential(
        alias_tuple.id_alias,
        credential_spec,
        credential,
    ))
}

/// Internal parameters to pass to the build_credential_jwt function.
struct CredentialParams {
    spec: CredentialSpec,
    subject_id: String,
    credential_id: String,
    context: Vec<String>,
    issuer: String,
    claims: Vec<Claim>,
    expiration_timestamp_s: u32,
}

fn build_credential(
    subject_principal: Principal,
    credential_spec: &CredentialSpec,
    credential: StoredCredential,
) -> String {
    // Retrieve the context and issuer url from the LookupTable
    LOOKUP_TABLE.with(|map| {
        let map = map.borrow();
        let (issuer, context) = map
            .get(credential.context_issuer_id)
            .unwrap()
            .to_owned();
        let params = CredentialParams {
            spec: credential_spec.clone(),
            subject_id: did_for_principal(subject_principal),
            credential_id: credential.id,
            context,
            issuer: issuer,
            expiration_timestamp_s: exp_timestamp_s(),
            claims: credential.claim,
        };
        build_credential_jwt(params)
    })
}

fn exp_timestamp_s() -> u32 {
    ((time() + VC_EXPIRATION_PERIOD_NS) / 1_000_000_000) as u32
}

/// Build a VC and return it as a JWT-string.
fn build_credential_jwt(params: CredentialParams) -> String {
    // Build "credentialSubject" objects
    let subjects = build_claims_into_credential_subjects(params.claims, params.subject_id);
    let expiration_date = Timestamp::from_unix(params.expiration_timestamp_s as i64)
        .expect("internal: failed computing expiration timestamp");

    // Build the VC a
    let mut credential = CredentialBuilder::default()
        .id(Url::parse(params.credential_id).unwrap())
        .issuer(Url::parse(params.issuer).unwrap())
        .type_("VerifiedCredential".to_string())
        .type_(params.spec.credential_type)
        .subjects(subjects) // add objects to the credentialSubject
        .expiration_date(expiration_date);
    // Add all the context
    credential = add_context(credential, params.context);

    // Serialize the VC object into a JWT-string
    let credential = credential.build().unwrap();
    credential.serialize_jwt().unwrap()
}

/// Helper function to construct the claims stored in the canister into a CredentialSubject containing the subject and given claims
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::Claim;
    use std::collections::HashMap;

    /// Test that new entry is added to the table if there doesn't exist one for the given values
    #[test]
    fn test_add_new_entry_to_table() {
        let full_credential = FullCredential {
            id: "http://example.com/credentials/123".to_string(),
            type_: vec!["VerifiedCredential".to_string()],
            issuer: "https://www.civic.com".to_string(),
            context: vec![
                "https://www.w3.org/ns/credentials/v2".to_string(),
                "https://www.example.com/credentials/extension".to_string(),
            ],
            claim: vec![Claim {
                claims: HashMap::new(),
            }],
        };

        let stored_credential = StoredCredential::from(full_credential);
        assert_eq!(stored_credential.context_issuer_id, 1);
        assert_eq!(
            LOOKUP_TABLE.with_borrow(|t| t.get(1).unwrap().0.clone()),
            ("https://www.civic.com".to_string())
        );
    }

    /// Test that the table will use existing entries if applicable  
    #[test]
    fn test_use_existing_entry_in_table() {
        // Add the entry to the table
        test_add_new_entry_to_table();
        // Create a new full credential with the same url and context as the one added above
        let full_credential = FullCredential {
            id: "http://example.com/credentials/123".to_string(),
            type_: vec!["VerifiedCredential".to_string()],
            issuer: "https://www.civic.com".to_string(),
            context: vec![
                "https://www.w3.org/ns/credentials/v2".to_string(),
                "https://www.example.com/credentials/extension".to_string(),
            ],
            claim: vec![Claim {
                claims: HashMap::new(),
            }],
        };
        // Convert the credential and verify the context_issuer_id remains the same
        let stored_credential = StoredCredential::from(full_credential);
        assert_eq!(stored_credential.context_issuer_id, 1);
    }

    /// Test that the conversion from FullCredential to StoredCredential works as expected
    #[test]
    fn test_convert_full_credential_to_stored_credential() {
        let full_credential = FullCredential {
            id: "http://example.com/credentials/123".to_string(),
            type_: vec!["VerifiedCredential".to_string()],
            issuer: "https://www.civic.com".to_string(),
            context: vec![
                "https://www.w3.org/ns/credentials/v2".to_string(),
                "https://www.example.com/credentials/extension".to_string(),
            ],
            claim: vec![Claim {
                claims: HashMap::new(),
            }],
        };
        let stored_credential = StoredCredential::from(full_credential);
        assert_eq!(stored_credential.context_issuer_id, 1);
    }

    /// Test conversion from StoredCredential to FullCredential (only implemented for an array)
    #[test]
    fn test_convert_list_of_stored_credential_to_list_of_full_credential() {
        // Create and compress multiple full credentials to populate the lookup table
        let credential1 = FullCredential {
            id: "http://example.com/credentials/123".to_string(),
            type_: vec!["VerifiedCredential".to_string()],
            issuer: "https://www.civic.com".to_string(),
            context: vec![
                "https://www.w3.org/ns/credentials/v2".to_string(),
                "https://www.example.com/credentials/extension".to_string(),
            ],
            claim: vec![Claim {
                claims: HashMap::new(),
            }],
        };

        let credential2 = FullCredential {
            id: "http://example.com/credentials/123".to_string(),
            type_: vec!["VerifiedCredential".to_string()],
            issuer: "https://www.civic.com/issuer".to_string(),
            context: vec![
                "https://www.w3.org/ns/credentials/v2".to_string(),
                "https://www.example.com/credentials/extension".to_string(),
            ],
            claim: vec![Claim {
                claims: HashMap::new(),
            }],
        };
        // Convert them so that the table will be filled with entries for '1' and '2'
        let c1 = StoredCredential::from(credential1);
        let c2 = StoredCredential::from(credential2);

        // Convert them back to FullCredential and check if the values are correct
        let full_credentials: Vec<FullCredential> = CredentialList(vec![c1, c2]).into();

        assert_eq!(
            full_credentials[0].issuer,
            "https://www.civic.com".to_string()
        );
        assert_eq!(
            full_credentials[0].context[0],
            "https://www.w3.org/ns/credentials/v2".to_string()
        );
        assert_eq!(
            full_credentials[0].context[1],
            "https://www.example.com/credentials/extension".to_string()
        );
        assert_eq!(
            full_credentials[1].issuer,
            "https://www.civic.com/issuer".to_string()
        );
        assert_eq!(
            full_credentials[1].context[0],
            "https://www.w3.org/ns/credentials/v2".to_string()
        );
        assert_eq!(
            full_credentials[1].context[1],
            "https://www.example.com/credentials/extension".to_string()
        );
    }
}
