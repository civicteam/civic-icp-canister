pub mod types;
use types::{Claim, StoredCredential, CredentialError, build_claims_into_credentialSubjects, add_context};

use std::fmt;
use candid::{candid_method, CandidType, Deserialize, Principal};
// use ic_cdk::candid::candid_method;
use canister_sig_util::{extract_raw_root_pk_from_der, CanisterSigPublicKey, IC_ROOT_PK_DER};
use canister_sig_util::signature_map::{SignatureMap, LABEL_SIG};

use ic_cdk::api::{caller, set_certified_data, time};
use ic_cdk_macros::{init, query, update};
use ic_certification::{fork_hash, labeled_hash, pruned, Hash};

use std::collections::{HashSet,HashMap};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, RestrictedMemory, StableCell, Storable};
use include_dir::{include_dir, Dir};
use sha2::{Digest, Sha256};

use serde_bytes::ByteBuf;
use serde::{ Serialize};
use serde_json::{Value as JsonValue, json};
use std::borrow::Cow;
use std::cell::RefCell;
use asset_util::{collect_assets, CertifiedAssets};
use vc_util::issuer_api::{
    ArgumentValue, CredentialSpec, DerivationOriginData, DerivationOriginError,
    DerivationOriginRequest, GetCredentialRequest, Icrc21ConsentInfo, Icrc21Error,
    Icrc21VcConsentMessageRequest, IssueCredentialError, IssuedCredentialData,
    PrepareCredentialRequest, PreparedCredentialData, SignedIdAlias,
};
use vc_util::{ did_for_principal, get_verified_id_alias_from_jws, vc_jwt_to_jws,
    vc_signing_input, vc_signing_input_hash, AliasTuple,
};
use ic_cdk::api;
use lazy_static::lazy_static;
use identity_credential::credential::{self, Credential, CredentialBuilder, Jwt, Subject};
use identity_core::common::{Timestamp, Url, Context};


/// We use restricted memory in order to ensure the separation between non-managed config memory (first page)
/// and the managed memory for potential other data of the canister.
type Memory = RestrictedMemory<DefaultMemoryImpl>;
type ConfigCell = StableCell<IssuerConfig, Memory>;

// The expiration of issued verifiable credentials.
const MINUTE_NS: u64 = 60 * 1_000_000_000;
const PROD_II_CANISTER_ID: &str = "rdmx6-jaaaa-aaaaa-aaadq-cai";
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
            SupportedCredentialType::VerifiedAdult => write!(f, "VerifiedEmployee"),
        }
    }
}


thread_local! {
    /// Static configuration of the canister set by init() or post_upgrade().
    static CONFIG: RefCell<ConfigCell> = RefCell::new(ConfigCell::init(config_memory(), IssuerConfig::default()).expect("failed to initialize stable cell"));
    static SIGNATURES : RefCell<SignatureMap> = RefCell::new(SignatureMap::default());

    static ADULTS : RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());
    static CREDENTIALS : RefCell<HashMap<Principal, Vec<StoredCredential>>> = RefCell::new(HashMap::new());
    // Assets for the management app
    static ASSETS: RefCell<CertifiedAssets> = RefCell::new(CertifiedAssets::default());
}


/// Reserve the first stable memory page for the configuration stable cell.
fn config_memory() -> Memory {
    RestrictedMemory::new(DefaultMemoryImpl::default(), 0..1)
}


#[cfg(target_arch = "wasm32")]
use ic_cdk::println;

#[derive(CandidType, Deserialize)]
struct IssuerConfig {
    /// Root of trust for checking canister signatures.
    ic_root_key_raw: Vec<u8>,
    /// List of canister ids that are allowed to provide id alias credentials.
    idp_canister_ids: Vec<Principal>,
    /// The derivation origin to be used by the issuer.
    derivation_origin: String,
    /// Frontend hostname to be used by the issuer.
    frontend_hostname: String,
}

impl Storable for IssuerConfig {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).expect("failed to encode IssuerConfig"))
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).expect("failed to decode IssuerConfig")
    }
    const BOUND: Bound = Bound::Unbounded;
}

impl Default for IssuerConfig {
    fn default() -> Self {
        let derivation_origin = format!("https://{}.icp0.io", ic_cdk::id().to_text());
        Self {
            ic_root_key_raw: extract_raw_root_pk_from_der(IC_ROOT_PK_DER)
                .expect("failed to extract raw root pk from der"),
            idp_canister_ids: vec![Principal::from_text(PROD_II_CANISTER_ID).unwrap()],
            derivation_origin: derivation_origin.clone(),
            frontend_hostname: derivation_origin, // by default, use DERIVATION_ORIGIN as frontend-hostname
        }
    }
}


impl From<IssuerInit> for IssuerConfig {
    fn from(init: IssuerInit) -> Self {
        Self {
            ic_root_key_raw: extract_raw_root_pk_from_der(&init.ic_root_key_der)
                .expect("failed to extract raw root pk from der"),
            idp_canister_ids: init.idp_canister_ids,
            derivation_origin: init.derivation_origin,
            frontend_hostname: init.frontend_hostname,
        }
    }
}

#[derive(CandidType, Deserialize)]
struct IssuerInit {
    /// Root of trust for checking canister signatures.
    ic_root_key_der: Vec<u8>,
    /// List of canister ids that are allowed to provide id alias credentials.
    idp_canister_ids: Vec<Principal>,
    /// The derivation origin to be used by the issuer.
    derivation_origin: String,
    /// Frontend hostname be used by the issuer.
    frontend_hostname: String,
}

#[init]
#[candid_method(init)]
fn init(init_arg: Option<IssuerInit>) {
    if let Some(init) = init_arg {
        apply_config(init);
    };

    init_assets();
}

// #[post_upgrade]
// fn post_upgrade(init_arg: Option<IssuerInit>) {
//     init(init_arg);
// }

#[update]
#[candid_method]
fn configure(config: IssuerInit) {
    apply_config(config);
}

fn apply_config(init: IssuerInit) {
    CONFIG
        .with_borrow_mut(|config_cell| config_cell.set(IssuerConfig::from(init)))
        .expect("failed to apply issuer config");
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


fn update_root_hash() {
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
    let credential_jwt: String = match String::from_utf8(prepared_context.into_vec()){
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


fn static_headers() -> Vec<HeaderField> {
    vec![("Access-Control-Allow-Origin".to_string(), "*".to_string())]
}


fn internal_error(msg: &str) -> IssueCredentialError {
    IssueCredentialError::Internal(String::from(msg))
}
// Assets
static ASSET_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/dist");
pub fn init_assets() {
    ASSETS.with_borrow_mut(|assets| {
        *assets = CertifiedAssets::certify_assets(
            collect_assets(&ASSET_DIR, Some(fixup_html)),
            &static_headers(),
        );
    });

    update_root_hash()
}
pub type HeaderField = (String, String);

fn fixup_html(html: &str) -> String {
    let canister_id = api::id();

    // the string we are replacing here is inserted by vite during the front-end build
    html.replace(
            r#"<script type="module" crossorigin src="/index.js"></script>"#,
            &format!(r#"<script data-canister-id="{canister_id}" type="module" crossorigin src="/index.js"></script>"#).to_string()
        )
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
        credential
    ))

}

fn build_credential(
    subject_principal: Principal,
    credential_spec: &CredentialSpec,
    credential: StoredCredential
) -> String {
    let params = CredentialParams {
        spec: credential_spec.clone(),
        subject_id: did_for_principal(subject_principal),
        credential_id_url: credential.id,
        context: credential.context,
        issuer_url: credential.issuer,
        expiration_timestamp_s: exp_timestamp_s(),
        claims: vec!(Claim{claims: HashMap::new()}),
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
                return Ok(c)
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


fn verify_credential_spec(spec: &CredentialSpec) -> Result<SupportedCredentialType, String> {
    match spec.credential_type.as_str() {
        "VerifiedAdult" => {
            Ok(SupportedCredentialType::VerifiedAdult)
        }
        other => Err(format!("Credential {} is not supported", other)),
    }
}


#[update]
#[candid_method]
fn add_credentials(principal: Principal, new_credentials: Vec<StoredCredential>) -> String {
    CREDENTIALS.with_borrow_mut(|credentials| {
            let entry = credentials.entry(principal).or_insert_with(Vec::new);
            entry.extend(new_credentials);    
        });
    format!("Added credentials")
}



#[query]
#[candid_method(query)]
fn get_all_credentials(principal: Principal) -> Result<Vec<StoredCredential>, CredentialError> {
    if let Some(c) = CREDENTIALS.with_borrow(|credentials| {
        credentials.get(&principal).cloned()
    }) {
        Ok(c)
    } else {
        Err(CredentialError::NoCredentialsFound(format!("No credentials found for principal {}", principal.to_text())))
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


 struct CredentialParams {
     spec: CredentialSpec,
     subject_id: String,
     credential_id_url: String, 
     context: Vec<String>,
     issuer_url: String,
     claims: Vec<Claim>,
     expiration_timestamp_s: u32,
}


/// Builds a verifiable credential with the given parameters and returns the credential as a JWT-string.
pub fn build_credential_jwt(params: CredentialParams) -> String {
    // let mut subject_json = json!({"id": params.subject_id});
    // subject_json.as_object_mut().unwrap().insert(
    //     params.spec.credential_type.clone(),
    //     credential_spec_args_to_json(&params.spec),
    // );
    // let subject = Subject::from_json_value(subject_json).unwrap();

    // build "credentialSubject" objects
    let subjects = build_claims_into_credentialSubjects(params.claims, params.subject_id); 
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
    // //add all the type data 
    // credential = add_types(credential, params.types_);

    let credential = credential.build().unwrap();
    credential.serialize_jwt().unwrap()
}

ic_cdk::export_candid!();


// candid::export_service!();

// #[cfg(test)]
// mod test {
//     use crate::__export_service;
//     use candid_parser::utils::{service_equal, CandidSource};
//     use std::path::Path;

//     /// Checks candid interface type equality by making sure that the service in the did file is
//     /// a subtype of the generated interface and vice versa.
//     #[test]
//     fn check_candid_interface_compatibility() {
//         let canister_interface = __export_service();
//         service_equal(
//             CandidSource::Text(&canister_interface),
//             CandidSource::File(Path::new("civic_canister_backend.did")),
//         )
//         .unwrap_or_else(|e| {
//             panic!(
//                 "the canister code interface is not equal to the did file: {:?}",
//                 e
//             )
//         });
//     }
// }