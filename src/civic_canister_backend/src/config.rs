//! Configuration management for the Civic Canister
//!
//! This module handles:
//! - Initialization and configuration of the canister settings.
//! - Storing and updating issuer configurations.
//! - Managing assets and their certification.
//! - Handling HTTP requests with CORS support.

use crate::credential::{update_root_hash, CredentialList, CANISTER_SIG_SEED};
use asset_util::{collect_assets, CertifiedAssets};
use candid::{candid_method, CandidType, Deserialize, Principal};
use canister_sig_util::signature_map::{SignatureMap, LABEL_SIG};
use canister_sig_util::{extract_raw_root_pk_from_der, IC_ROOT_PK_DER};
use ic_cdk::api;
use ic_cdk_macros::{init, post_upgrade, query, update};
use ic_certification::{labeled_hash, pruned};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, StableCell, StableVec, Storable,
};
use include_dir::{include_dir, Dir};
use serde_bytes::ByteBuf;
use std::borrow::Cow;
use std::cell::RefCell;
use vc_util::issuer_api::{DerivationOriginData, DerivationOriginError, DerivationOriginRequest};

const PROD_II_CANISTER_ID: &str = "rdmx6-jaaaa-aaaaa-aaadq-cai";

// A memory for config, where data from the heap can be serialized/deserialized.
const CONF: MemoryId = MemoryId::new(0);
// A memory for Signatures, where data from the heap can be serialized/deserialized.
const SIG: MemoryId = MemoryId::new(1);

// A memory for the Credential data
const CREDENTIAL: MemoryId = MemoryId::new(2);

type ConfigCell = StableCell<IssuerConfig, VirtualMemory<DefaultMemoryImpl>>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub(crate) static CONFIG: RefCell<ConfigCell> = RefCell::new(ConfigCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(CONF)), IssuerConfig::default()).expect("failed to initialize stable cell"));
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
}

#[cfg(target_arch = "wasm32")]
use ic_cdk::println;

/// Configuration for the canister.
#[derive(Clone, Debug, CandidType, Deserialize)]
pub(crate) struct IssuerConfig {
    /// Root of trust for checking canister signatures.
    pub(crate) ic_root_key_raw: Vec<u8>,
    /// List of canister ids that are allowed to provide id alias credentials.
    pub(crate) idp_canister_ids: Vec<Principal>,
    /// The derivation origin to be used by the issuer.
    derivation_origin: String,
    /// Frontend hostname to be used by the issuer.
    frontend_hostname: String,
    // Admin who can add authorized issuers
    admin: Principal,
    // List of authorized issuers who can issue credentials
    pub authorized_issuers: Vec<Principal>,
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
            frontend_hostname: derivation_origin,
            admin: ic_cdk::api::caller(),
            authorized_issuers: vec![ic_cdk::api::caller()],
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
            admin: init.admin,
            authorized_issuers: init.authorized_issuers,
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
    // Admin who can add authorized issuers
    admin: Principal,
    // List of authorized issuers who can issue credentials
    authorized_issuers: Vec<Principal>,
}

/// Called when the canister is deployed.
#[init]
#[candid_method(init)]
fn init(init_arg: Option<IssuerInit>) {
    if let Some(init) = init_arg {
        apply_config(init);
    } else {
        // Initialize with default values and a specified admin
        let default_config = IssuerConfig::default();
        CONFIG.with(|config_cell| {
            let mut config = config_cell.borrow_mut();
            *config = ConfigCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(CONF)), default_config)
                .expect("Failed to initialize config");
        });
    }
    init_assets();
}

#[update]
#[candid_method(update)]
fn add_issuer(new_issuer: Principal) {
    let caller = ic_cdk::api::caller();
    CONFIG.with(|config_cell| {
        let mut config = config_cell.borrow_mut();
        // Retrieve the current configuration
        let mut current_config = config.get().clone(); // Clone into a mutable local variable

        // Check if the caller is the admin and modify the config
        if caller == current_config.admin {
            // Ensure no duplicates if that's intended
            if !current_config.authorized_issuers.contains(&new_issuer) {
                current_config.authorized_issuers.push(new_issuer);
            }
            // Save the updated configuration
            let _ = config.set(current_config); // Pass the modified IssuerConfig back to set
        } else {
            ic_cdk::api::trap("Caller is not authorized as admin.");
        }
    });
}

#[update]
#[candid_method(update)]
fn remove_issuer(issuer: Principal) {
    let caller = ic_cdk::api::caller();
    CONFIG.with(|config_cell| {
        let mut config = config_cell.borrow_mut();
        // Retrieve the current configuration
        let mut current_config = config.get().clone(); // Clone into a mutable local variable

        if caller == current_config.admin {
            // Remove the issuer if they exist in the list
            current_config.authorized_issuers.retain(|x| *x != issuer);
            // Save the updated configuration
            let _ = config.set(current_config); // Pass the modified IssuerConfig back to set
        } else {
            ic_cdk::api::trap("Caller is not authorized as admin.");
        }
    });
}

#[query]
fn get_admin() -> Principal {
    CONFIG.with(|config| {
        let config_borrowed = config.borrow(); // Obtain a read-only borrow
                                               // Now you can access the config
        return config_borrowed.get().admin;
    })
}

/// Called when the canister is upgraded.
#[post_upgrade]
fn post_upgrade(init_arg: Option<IssuerInit>) {
    // Initialize the CONFIG
    init(init_arg);

    // Restore the signatures
    SIGNATURES.with(|sigs| {
        let mut sigs = sigs.borrow_mut();
        MSG_HASHES.with(|hashes| {
            hashes.borrow().iter().for_each(|hash| {
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

ic_cdk::export_candid!();
