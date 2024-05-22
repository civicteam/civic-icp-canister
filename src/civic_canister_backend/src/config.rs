//! Configuration management for the Civic Canister
//!
//! This module handles:
//! - Initialization and configuration of the canister settings.
//! - Storing and updating issuer configurations.
//! - Managing assets and their certification.
//! - Handling HTTP requests with CORS support.

use std::cell::RefCell;
use canister_sig_util::signature_map::{SignatureMap, LABEL_SIG};
use candid::{candid_method, CandidType, Deserialize, Principal};
use canister_sig_util::{extract_raw_root_pk_from_der, IC_ROOT_PK_DER};
use ic_cdk_macros::{init, query, update, post_upgrade};
use candid_parser::{Encode, Decode};
use ic_cdk::api;
use ic_certification::{labeled_hash, pruned};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, RestrictedMemory, StableVec, StableCell, Storable, StableBTreeMap, memory_manager::{MemoryManager, MemoryId, VirtualMemory}};
use include_dir::{include_dir, Dir};
use serde_bytes::ByteBuf;
use std::borrow::Cow;
use asset_util::{collect_assets, CertifiedAssets};
use vc_util::issuer_api::{
    DerivationOriginData, DerivationOriginError,
    DerivationOriginRequest
};
use crate::credential::{CredentialList, update_root_hash, CANISTER_SIG_SEED};


const PROD_II_CANISTER_ID: &str = "rdmx6-jaaaa-aaaaa-aaadq-cai";

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const SIG: MemoryId = MemoryId::new(1);

// A memory for the StableBTreeMap we're using. A new memory should be created for
// every additional stable structure
const CREDENTIAL: MemoryId = MemoryId::new(2);

const MAX_VALUE_SIZE: u32 = 100;

#[derive(CandidType, Deserialize)]
struct UserProfile {
    age: u8,
    name: String,
}

impl Storable for UserProfile {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub(crate) static CONFIG: RefCell<ConfigCell> = RefCell::new(ConfigCell::init(config_memory(), IssuerConfig::default()).expect("failed to initialize stable cell"));
    pub(crate) static SIGNATURES : RefCell<SignatureMap> = RefCell::new(SignatureMap::default());

    pub(crate) static CREDENTIALS: RefCell<StableBTreeMap<Principal, CredentialList, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(CREDENTIAL))
        )
    );
    // Assets for the management app
    pub(crate) static ASSETS: RefCell<CertifiedAssets> = RefCell::new(CertifiedAssets::default());

    
    // Stable vector to restore the signatures when the canister is upgraded
    pub(crate) static MSG_HASHES: RefCell<StableVec<[u8; 32], VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableVec::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(SIG))
        ).expect("failed to initialize stable vector")
    );

        static MAP: RefCell<StableBTreeMap<u64, UserProfile, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );

}

/// We use restricted memory in order to ensure the separation between non-managed config memory (first page)
/// and the managed memory for the credential data of the canister.
type Memory= RestrictedMemory<DefaultMemoryImpl>;
type ConfigCell = StableCell<IssuerConfig, Memory>;


/// Reserve the first stable memory page for the configuration stable cell.
fn config_memory() -> Memory{
    RestrictedMemory::new(DefaultMemoryImpl::default(), 0..1)
}

#[cfg(target_arch = "wasm32")]
use ic_cdk::println;

/// Configuration for the canister.
#[derive(CandidType, Deserialize)]
pub(crate) struct IssuerConfig {
    /// Root of trust for checking canister signatures.
    pub(crate) ic_root_key_raw: Vec<u8>,
    /// List of canister ids that are allowed to provide id alias credentials.
    pub(crate) idp_canister_ids: Vec<Principal>,
    /// The derivation origin to be used by the issuer.
    derivation_origin: String,
    /// Frontend hostname to be used by the issuer.
    frontend_hostname: String,
}

impl Storable for IssuerConfig {
    fn to_bytes(&self) -> Cow<[u8]> {
        ic_cdk::print("serializing config");
        Cow::Owned(candid::encode_one(self).expect("failed to encode IssuerConfig"))
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        ic_cdk::print("deserializing config");
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
            frontend_hostname: derivation_origin,
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

/// Initialization arguments for the canister.
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

/// Called when the canister is deployed.
#[init]
#[candid_method(init)]
fn init(init_arg: Option<IssuerInit>) {
    if let Some(init) = init_arg {
        apply_config(init);
    };

    init_assets();
}

/// A post-upgrade hook for configuring the canister and deserializing the data back into the heap.
#[post_upgrade]
fn post_upgrade(init_arg: Option<IssuerInit>) {
    // Initialize the CONFIG
    init(init_arg);

    // Restore the signatures
    SIGNATURES.with(|sigs| {    
        let mut sigs = sigs.borrow_mut();
        MSG_HASHES.with(|hashes| {
        hashes.borrow().iter().for_each(|hash| {
            ic_cdk::print("adding signature ");
            sigs.add_signature(&CANISTER_SIG_SEED, hash);
        })
        });
    });

    update_root_hash();
}

/// Called when the canister is configured.
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

fn static_headers() -> Vec<HeaderField> {
    vec![("Access-Control-Allow-Origin".to_string(), "*".to_string())]
}

/// Assets
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

/// Get the derivation origin used by the canister
#[update]
#[candid_method]
async fn derivation_origin(
    req: DerivationOriginRequest,
) -> Result<DerivationOriginData, DerivationOriginError> {
    get_derivation_origin(&req.frontend_hostname)
}

fn get_derivation_origin(hostname: &str) -> Result<DerivationOriginData, DerivationOriginError> {
    CONFIG.with_borrow(|config| {
        let config = config.get();

        // We don't currently rely on the value provided, so if it doesn't match
        // we just print a warning
        if hostname != config.frontend_hostname {
            println!("*** achtung! bad frontend hostname {}", hostname,);
        }

        Ok(DerivationOriginData {
            origin: config.derivation_origin.clone(),
        })
    })
}

/// Handle HTTP requests with CORS support.
#[query]
#[candid_method(query)]
pub fn http_request(req: HttpRequest) -> HttpResponse {
    let parts: Vec<&str> = req.url.split('?').collect();
    let path = parts[0];
    let sigs_root_hash =
        SIGNATURES.with_borrow(|sigs| pruned(labeled_hash(LABEL_SIG, &sigs.root_hash())));
    let maybe_asset = ASSETS.with_borrow(|assets| {
        assets.get_certified_asset(path, req.certificate_version, Some(sigs_root_hash))
    });

    let mut headers = static_headers();
    match maybe_asset {
        Some(asset) => {
            headers.extend(asset.headers);
            HttpResponse {
                status_code: 200,
                body: ByteBuf::from(asset.content),
                headers,
            }
        }
        None => HttpResponse {
            status_code: 404,
            headers,
            body: ByteBuf::from(format!("Asset {} not found.", path)),
        },
    }
}


#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
    pub certificate_version: Option<u16>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
}


#[ic_cdk_macros::query]
fn get(key: u64) -> Option<UserProfile> {
    MAP.with(|p| p.borrow().get(&key))
}

#[ic_cdk_macros::update]
fn insert(key: u64, value: UserProfile) -> Option<UserProfile> {
    MAP.with(|p| p.borrow_mut().insert(key, value))
}

ic_cdk::export_candid!();
