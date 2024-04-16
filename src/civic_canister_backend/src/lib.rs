use candid::types::principal;
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

use serde_bytes::ByteBuf;
use serde::{ Serialize};
use serde_json::Value as JsonValue;

use std::borrow::Cow;
use std::cell::RefCell;
use asset_util::{collect_assets, CertifiedAssets};
use vc_util::issuer_api::{
    ArgumentValue, CredentialSpec, DerivationOriginData, DerivationOriginError,
    DerivationOriginRequest, GetCredentialRequest, Icrc21ConsentInfo, Icrc21Error,
    Icrc21VcConsentMessageRequest, IssueCredentialError, IssuedCredentialData,
    PrepareCredentialRequest, PreparedCredentialData, SignedIdAlias,
};
use vc_util::{
    build_credential_jwt, did_for_principal, get_verified_id_alias_from_jws, vc_jwt_to_jws,
    vc_signing_input, vc_signing_input_hash, AliasTuple, CredentialParams,
};
use ic_cdk::api;


/// We use restricted memory in order to ensure the separation between non-managed config memory (first page)
/// and the managed memory for potential other data of the canister.
type Memory = RestrictedMemory<DefaultMemoryImpl>;
type ConfigCell = StableCell<IssuerConfig, Memory>;

// const MINUTE_NS: u64 = 60 * 1_000_000_000;
// // The expiration of issued verifiable credentials.
// const VC_EXPIRATION_PERIOD_NS: u64 = 15 * MINUTE_NS;
const PROD_II_CANISTER_ID: &str = "rdmx6-jaaaa-aaaaa-aaadq-cai";

#[derive( CandidType, Serialize, Deserialize, Debug, Clone)]
struct CredentialSubject {
    additional_data: JsonValue, 
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
enum ClaimValue {
    Boolean(bool),
    Date(String),
    Text(String),
    CredentialSubjects(Vec<CredentialSubject>), 
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
struct Claim {
    key: String,
    value: ClaimValue,
}

#[derive( Serialize, Deserialize, Debug)]
struct Credential {
    id: String, 
    type_: Vec<String>,
    context: Vec<String>,
    issuer: String,
    claim: Vec<Claim>,
}
#[derive(CandidType)]
enum CredentialError {
    NoClaimsFound(String),
}

#[derive(Debug)]
pub enum SupportedCredentialType {
    VerifiedAdult,
}

thread_local! {
    /// Static configuration of the canister set by init() or post_upgrade().
    static CONFIG: RefCell<ConfigCell> = RefCell::new(ConfigCell::init(config_memory(), IssuerConfig::default()).expect("failed to initialize stable cell"));
    static SIGNATURES : RefCell<SignatureMap> = RefCell::new(SignatureMap::default());

    static ADULTS : RefCell<HashSet<Principal>> = RefCell::new(HashSet::new());
    static CLAIMS : RefCell<HashMap<Principal, Vec<Claim>>> = RefCell::new(HashMap::new());
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

#[query]
#[candid_method]
async fn whoami() -> Principal {
    caller()
}

#[update]
#[candid_method]
async fn prepare_credential(
    req: PrepareCredentialRequest,
) -> Result<PreparedCredentialData, IssueCredentialError> {
    let alias_tuple = match authorize_vc_request(&req.signed_id_alias, &caller(), time().into()) {
        Ok(alias_tuple) => alias_tuple,
        Err(err) => return Err(err),
    };


    let credential_string = match prepare_credential_string(&req.credential_spec, &alias_tuple) {
        Ok(credential) => credential,
        Err(err) => return Result::<PreparedCredentialData, IssueCredentialError>::Err(err),
    };
    // let signing_input =
    //     vc_signing_input(&credential_jwt, &CANISTER_SIG_PK).expect("failed getting signing_input");
    // let msg_hash = vc_signing_input_hash(&signing_input);

    // SIGNATURES.with(|sigs| {
    //     let mut sigs = sigs.borrow_mut();
    //     sigs.add_signature(&CANISTER_SIG_SEED, msg_hash);
    // });
    // update_root_hash();

    // return a prepared context 
    Ok(PreparedCredentialData {
        prepared_context: Some(ByteBuf::from(credential_string.as_bytes())),
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
    let credential_string = match String::from_utf8(prepared_context.into_vec()){
    // let credential_jwt = match String::from_utf8(prepared_context.into_vec()) {
        Ok(s) => s,
        Err(_) => {
            return Result::<IssuedCredentialData, IssueCredentialError>::Err(internal_error(
                "invalid prepared_context",
            ))
        }
    };
    // let signing_input =
    //     vc_signing_input(&credential_jwt, &CANISTER_SIG_PK).expect("failed getting signing_input");
    // let message_hash = vc_signing_input_hash(&signing_input);
    // let sig_result = SIGNATURES.with(|sigs| {
    //     let sig_map = sigs.borrow();
    //     let certified_assets_root_hash = ASSETS.with_borrow(|assets| assets.root_hash());
    //     sig_map.get_signature_as_cbor(
    //         &CANISTER_SIG_SEED,
    //         message_hash,
    //         Some(certified_assets_root_hash),
    //     )
    // });
    // let sig = match sig_result {
    //     Ok(sig) => sig,
    //     Err(e) => {
    //         return Result::<IssuedCredentialData, IssueCredentialError>::Err(
    //             IssueCredentialError::SignatureNotFound(format!(
    //                 "signature not prepared or expired: {}",
    //                 e
    //             )),
    //         );
    //     }
    // };

    // let vc_jws =
    //     vc_jwt_to_jws(&credential_jwt, &CANISTER_SIG_PK, &sig).expect("failed constructing JWS");
    // Result::<IssuedCredentialData, IssueCredentialError>::Ok(IssuedCredentialData { vc_jws })
    Result::<IssuedCredentialData, IssueCredentialError>::Ok(IssuedCredentialData { vc_jws: credential_string })
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

// this issues a simple string credential to the user alias 
fn prepare_credential_string(
    credential_spec: &CredentialSpec,
    alias_tuple: &AliasTuple,
) -> Result<String, IssueCredentialError> {
    let credential_type = match verify_credential_spec(credential_spec) {
        Ok(credential_type) => credential_type,
        Err(err) => {
            return Err(IssueCredentialError::UnsupportedCredentialSpec(err));
        }
    };
    ADULTS.with_borrow(|adults| {
        verify_authorized_principal(credential_type, alias_tuple, adults)
    })?;
    Ok("Verified as being over 18: ".to_owned() + &alias_tuple.id_alias.to_string()
    )

}

// checks if the user has a credential, i.e. is contained in set of adults 
fn verify_authorized_principal(
    credential_type: SupportedCredentialType,
    alias_tuple: &AliasTuple,
    authorized_principals: &HashSet<Principal>,
) -> Result<(), IssueCredentialError> {
    if authorized_principals.contains(&alias_tuple.id_dapp) {
        Ok(())
    } else {
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
fn add_adult(adult_id: Principal) -> String {
    ADULTS.with_borrow_mut(|adults| adults.insert(adult_id));
    format!("Added adult {}", adult_id)
}


#[update]
#[candid_method]
fn add_claims(principal: Principal, claims: Vec<Claim>) -> String {
        CLAIMS.with_borrow_mut(|credentials| {
            let entry = credentials.entry(principal).or_insert_with(Vec::new);
            entry.extend(claims);    
        });
    format!("Added claims")
}



#[query]
#[candid_method(query)]
fn get_claims(principal: Principal) -> Result<Vec<Claim>, CredentialError> {
    let claims = CLAIMS.with_borrow(|claims| {
        claims.get(&principal).cloned()
    });
    if let Some(claims) = claims {
        Ok(claims)
    } else {
        Err(CredentialError::NoClaimsFound(format!("No claims found for principal {}", principal.to_text())))
    }
}
