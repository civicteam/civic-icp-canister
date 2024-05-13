//! Tests related to issue_credential canister call.

// use crate::types::{Claim, ClaimValue, StoredCredential};
use assert_matches::assert_matches;
use candid::{CandidType, Deserialize, Principal};
use canister_sig_util::{extract_raw_root_pk_from_der, CanisterSigPublicKey};

use canister_tests::api::internet_identity::vc_mvp as ii_api;
use canister_tests::flows;
use canister_tests::framework::{env, get_wasm_path, principal_1, test_principal, II_WASM};
use ic_cdk::api::management_canister::provisional::CanisterId;


use ic_test_state_machine_client::{call_candid, call_candid_as};
use ic_test_state_machine_client::{query_candid_as, CallError, StateMachine};


use internet_identity_interface::internet_identity::types::vc_mvp::{
    GetIdAliasRequest, PrepareIdAliasRequest,
};
use internet_identity_interface::internet_identity::types::FrontendHostname;
use lazy_static::lazy_static;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{UNIX_EPOCH};
use vc_util::issuer_api::{
    CredentialSpec, DerivationOriginData, DerivationOriginError,
    DerivationOriginRequest, GetCredentialRequest, Icrc21ConsentInfo,
    Icrc21Error, Icrc21VcConsentMessageRequest, IssueCredentialError, IssuedCredentialData,
    PrepareCredentialRequest, PreparedCredentialData, SignedIdAlias as SignedIssuerIdAlias,
};
use vc_util::{
    get_verified_id_alias_from_jws, verify_credential_jws_with_canister_id

};

// use crate::civic_canister_backend::types::{Claim, StoredCredential, CredentialError, ClaimValue, build_claims_into_credentialSubjects, add_context};

const DUMMY_ROOT_KEY: &str ="308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100adf65638a53056b2222c91bb2457b0274bca95198a5acbdadfe7fd72178f069bdea8d99e9479d8087a2686fc81bf3c4b11fe275570d481f1698f79d468afe0e57acc1e298f8b69798da7a891bbec197093ec5f475909923d48bfed6843dbed1f";
const DUMMY_II_CANISTER_ID: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
const DUMMY_DERIVATION_ORIGIN: &str = "https://y2aaj-miaaa-aaaad-aacxq-cai.ic0.app";
const DUMMY_FRONTEND_HOSTNAME: &str = "https://y2aaj-miaaa-aaaad-aacxq-cai.ic0.app";

/// Dummy alias JWS for testing, valid wrt DUMMY_ROOT_KEY and DUMMY_II_CANISTER_ID.
/// id dapp: nugva-s7c6v-4yszt-koycv-5b623-an7q6-ha2nz-kz6rs-hawgl-nznbe-rqe
/// id alias: jkk22-zqdxc-kgpez-6sv2m-5pby4-wi4t2-prmoq-gf2ih-i2qtc-v37ac-5ae
const DUMMY_ALIAS_JWS: &str ="eyJqd2siOnsia3R5Ijoib2N0IiwiYWxnIjoiSWNDcyIsImsiOiJNRHd3REFZS0t3WUJCQUdEdUVNQkFnTXNBQW9BQUFBQUFBQUFBQUVCMGd6TTVJeXFMYUhyMDhtQTRWd2J5SmRxQTFyRVFUX2xNQnVVbmN5UDVVYyJ9LCJraWQiOiJkaWQ6aWNwOnJ3bGd0LWlpYWFhLWFhYWFhLWFhYWFhLWNhaSIsImFsZyI6IkljQ3MifQ.eyJleHAiOjE2MjAzMjk1MzAsImlzcyI6Imh0dHBzOi8vaWRlbnRpdHkuaWMwLmFwcC8iLCJuYmYiOjE2MjAzMjg2MzAsImp0aSI6ImRhdGE6dGV4dC9wbGFpbjtjaGFyc2V0PVVURi04LHRpbWVzdGFtcF9uczoxNjIwMzI4NjMwMDAwMDAwMDAwLGFsaWFzX2hhc2g6YTI3YzU4NTQ0MmUwN2RkZWFkZTRjNWE0YTAzMjdkMzA4NTE5NDAzYzRlYTM3NDIxNzBhZTRkYzk1YjIyZTQ3MyIsInN1YiI6ImRpZDppY3A6bnVndmEtczdjNnYtNHlzenQta295Y3YtNWI2MjMtYW43cTYtaGEybnota3o2cnMtaGF3Z2wtbnpuYmUtcnFlIiwidmMiOnsiQGNvbnRleHQiOiJodHRwczovL3d3dy53My5vcmcvMjAxOC9jcmVkZW50aWFscy92MSIsInR5cGUiOlsiVmVyaWZpYWJsZUNyZWRlbnRpYWwiLCJJbnRlcm5ldElkZW50aXR5SWRBbGlhcyJdLCJjcmVkZW50aWFsU3ViamVjdCI6eyJJbnRlcm5ldElkZW50aXR5SWRBbGlhcyI6eyJoYXNJZEFsaWFzIjoiamtrMjItenFkeGMta2dwZXotNnN2Mm0tNXBieTQtd2k0dDItcHJtb3EtZ2YyaWgtaTJxdGMtdjM3YWMtNWFlIn19fX0.2dn3omtjZXJ0aWZpY2F0ZVkBsdnZ96JkdHJlZYMBgwGDAYMCSGNhbmlzdGVygwGDAkoAAAAAAAAAAAEBgwGDAYMBgwJOY2VydGlmaWVkX2RhdGGCA1ggvlJBTZDgK1_9Vb3-18dWKIfy28WTjZ1YqdjFWWAIX96CBFgg0sz_P8xdqTDewOhKJUHmWFFrS7FQHnDotBDmmGoFfWCCBFgg_KZ0TVqubo_EGWoMUPA35BYZ4B5ZRkR_zDfNIQCwa46CBFggj_ZV-7o59iVEjztzZtpNnO9YC7GjbKmg2eDtJzGz1weCBFggXAzCWvb9h4qsVs41IUJBABzjSqAZ8DIzF_ghGHpGmHGCBFggJhbsbvKYt7rjLK5SI0NDc600o-ajSYQNuOXps6qUrdiCBFggBFQwZetJeY_gx6TQohTqUOskblddajS20DA0esxWoyWDAYIEWCA1U_ZYHVOz3Sdkb2HIsNoLDDiBuFfG3DxH6miIwRPra4MCRHRpbWWCA0mAuK7U3YmkvhZpc2lnbmF0dXJlWDC5cq4UxYy7cnkcw6yv5SCh4POY9u0iHecZuxO8E9oxIqXRdHmnYVF0Fv_R-aws0EBkdHJlZYMBggRYIOGnlc_3yXPTVrEJ1p3dKX5HxkMOziUnpA1HeXiQW4O8gwJDc2lngwJYIIOQR7wl3Ws9Jb8VP4rhIb37XKLMkkZ2P7WaZ5we60WGgwGCBFgg21-OewBgqt_-0AtHHHS4yPyQK9g6JTHaGUuSIw4QYgqDAlgg5bQnHHvS3FfM_BaiSL6n19qoXkuA1KoLWk963fOUMW-CA0A";
const DUMMY_ALIAS_ID_DAPP_PRINCIPAL: &str =
    "nugva-s7c6v-4yszt-koycv-5b623-an7q6-ha2nz-kz6rs-hawgl-nznbe-rqe";

lazy_static! {
    /** The gzipped Wasm module for the current Civic_Canister_Backend build, i.e. the one we're testing */
    pub static ref CIVIV_CANISTER_BACKEND_WASM: Vec<u8> = {
        let def_path = PathBuf::from("./").join("civic_canister_backend.wasm.gz");
        let err = format!("
        Could not find VC Issuer Wasm module for current build.
        I will look for it at {:?} (note that I run from {:?}).
        ", &def_path,
            &std::env::current_dir().map(|x| x.display().to_string()).unwrap_or_else(|_|
                "an unknown directory".to_string()));
                get_wasm_path("CIVIV_CANISTER_BACKEND_WASM".to_string(), &def_path).expect(&err)

    };

    pub static ref DUMMY_ISSUER_INIT: IssuerInit = IssuerInit {
        ic_root_key_der: hex::decode(DUMMY_ROOT_KEY).unwrap(),
        idp_canister_ids: vec![Principal::from_text(DUMMY_II_CANISTER_ID).unwrap()],
        derivation_origin: DUMMY_DERIVATION_ORIGIN.to_string(),
        frontend_hostname: DUMMY_FRONTEND_HOSTNAME.to_string(),
    };

    pub static ref DUMMY_SIGNED_ID_ALIAS: SignedIssuerIdAlias = SignedIssuerIdAlias {
        credential_jws: DUMMY_ALIAS_JWS.to_string(),
    };
}

pub fn install_canister(env: &StateMachine, wasm: Vec<u8>) -> CanisterId {
    let canister_id = env.create_canister(None);
    let arg = candid::encode_one("()").expect("error encoding II installation arg as candid");
    env.install_canister(canister_id, wasm, arg, None);
    canister_id
}

#[derive(CandidType, Deserialize)]
pub struct IssuerInit {
    /// Root of trust for checking canister signatures.
    ic_root_key_der: Vec<u8>,
    /// List of canister ids that are allowed to provide id alias credentials.
    idp_canister_ids: Vec<Principal>,
    /// The derivation origin to be used by the issuer.
    derivation_origin: String,
    /// Frontend hostname be used by the issuer.
    frontend_hostname: String,
}

pub fn install_issuer(env: &StateMachine, init: &IssuerInit) -> CanisterId {
    let canister_id = env.create_canister(None);
    let arg = candid::encode_one(Some(init)).expect("error encoding II installation arg as candid");
    env.install_canister(canister_id, CIVIV_CANISTER_BACKEND_WASM.clone(), arg, None);
    canister_id
}

mod api {
    

    use super::*;

    pub fn configure(
        env: &StateMachine,
        canister_id: CanisterId,
        config: &IssuerInit,
    ) -> Result<(), CallError> {
        call_candid(env, canister_id, "configure", (config,))
    }

    pub fn vc_consent_message(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        consent_message_request: &Icrc21VcConsentMessageRequest,
    ) -> Result<Result<Icrc21ConsentInfo, Icrc21Error>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "vc_consent_message",
            (consent_message_request,),
        )
        .map(|(x,)| x)
    }

    pub fn derivation_origin(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        derivation_origin_req: &DerivationOriginRequest,
    ) -> Result<Result<DerivationOriginData, DerivationOriginError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "derivation_origin",
            (derivation_origin_req,),
        )
        .map(|(x,)| x)
    }

    pub fn add_adult(
        env: &StateMachine,
        canister_id: CanisterId,
        adult_id: Principal,
        credential: StoredCredential,
    ) -> Result<String, CallError> {
        let civic_issuer = Principal::from_text("tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae").unwrap();
        call_candid_as(env, canister_id, civic_issuer, "add_credentials", (adult_id, vec!(credential), )).map(|(x,)| x)
    }

    pub fn prepare_credential(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        prepare_credential_request: &PrepareCredentialRequest,
    ) -> Result<Result<PreparedCredentialData, IssueCredentialError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "prepare_credential",
            (prepare_credential_request,),
        )
        .map(|(x,)| x)
    }

    pub fn get_credential(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        get_credential_request: &GetCredentialRequest,
    ) -> Result<Result<IssuedCredentialData, IssueCredentialError>, CallError> {
        query_candid_as(
            env,
            canister_id,
            sender,
            "get_credential",
            (get_credential_request,),
        )
        .map(|(x,)| x)
    }
}

fn adult_credential_spec() -> CredentialSpec {
    // let mut args = HashMap::new();
    // args.insert("minAge".to_string(), ArgumentValue::Int(18));
    CredentialSpec {
        credential_type: "VerifiedAdult".to_string(),
        arguments: None,
    }
}

#[test]
fn should_fail_prepare_credential_for_unauthorized_principal() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let response = api::prepare_credential(
        &env,
        issuer_id,
        Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap(),
        &PrepareCredentialRequest {
            credential_spec: adult_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Err(e) if format!("{:?}", e).contains("unauthorized principal"));
}

#[test]
fn should_fail_prepare_credential_for_wrong_sender() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let signed_id_alias = DUMMY_SIGNED_ID_ALIAS.clone();

    let response = api::prepare_credential(
        &env,
        issuer_id,
        principal_1(), // not the same as contained in signed_id_alias
        &PrepareCredentialRequest {
            credential_spec: adult_credential_spec(),
            signed_id_alias,
        },
    )
    .expect("API call failed");
    assert_matches!(response,
        Err(IssueCredentialError::InvalidIdAlias(e)) if e.contains("id alias could not be verified")
    );
}

#[test]
fn should_fail_get_credential_for_wrong_sender() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let signed_id_alias = DUMMY_SIGNED_ID_ALIAS.clone();
    let authorized_principal = Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap();
    api::add_adult(&env, issuer_id, authorized_principal, construct_adult_credential()).expect("failed to add employee");
    let unauthorized_principal = test_principal(2);

    let prepare_credential_response = api::prepare_credential(
        &env,
        issuer_id,
        authorized_principal,
        &PrepareCredentialRequest {
            credential_spec: adult_credential_spec(),
            signed_id_alias: signed_id_alias.clone(),
        },
    )
    .expect("API call failed")
    .expect("failed to prepare credential");

    let get_credential_response = api::get_credential(
        &env,
        issuer_id,
        unauthorized_principal,
        &GetCredentialRequest {
            credential_spec: adult_credential_spec(),
            signed_id_alias,
            prepared_context: prepare_credential_response.prepared_context,
        },
    )
    .expect("API call failed");
    assert_matches!(get_credential_response,
        Err(IssueCredentialError::InvalidIdAlias(e)) if e.contains("id alias could not be verified")
    );
}

#[test]
fn should_fail_prepare_credential_for_anonymous_caller() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let response = api::prepare_credential(
        &env,
        issuer_id,
        Principal::anonymous(),
        &PrepareCredentialRequest {
            credential_spec: adult_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response,
        Err(IssueCredentialError::InvalidIdAlias(e)) if e.contains("id alias could not be verified")
    );
}

#[test]
fn should_fail_prepare_credential_for_wrong_root_key() {
    let env = env();
    let issuer_id = install_issuer(
        &env,
        &IssuerInit {
            ic_root_key_der: canister_sig_util::IC_ROOT_PK_DER.to_vec(), // does not match the DUMMY_ROOT_KEY, which is used in DUMMY_ALIAS_JWS
            idp_canister_ids: vec![Principal::from_text(DUMMY_II_CANISTER_ID).unwrap()],
            derivation_origin: DUMMY_DERIVATION_ORIGIN.to_string(),
            frontend_hostname: DUMMY_FRONTEND_HOSTNAME.to_string(),
        },
    );
    let response = api::prepare_credential(
        &env,
        issuer_id,
        Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap(),
        &PrepareCredentialRequest {
            credential_spec: adult_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Err(IssueCredentialError::InvalidIdAlias(_)));
}

#[test]
fn should_fail_prepare_credential_for_wrong_idp_canister_id() {
    let env = env();
    let issuer_id = install_issuer(
        &env,
        &IssuerInit {
            ic_root_key_der: hex::decode(DUMMY_ROOT_KEY).unwrap(),
            idp_canister_ids: vec![Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap()], // does not match the DUMMY_II_CANISTER_ID, which is used in DUMMY_ALIAS_JWS
            derivation_origin: DUMMY_DERIVATION_ORIGIN.to_string(),
            frontend_hostname: DUMMY_FRONTEND_HOSTNAME.to_string(),
        },
    );
    let response = api::prepare_credential(
        &env,
        issuer_id,
        Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap(),
        &PrepareCredentialRequest {
            credential_spec: adult_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Err(IssueCredentialError::InvalidIdAlias(_)));
}

#[test]
fn should_prepare_adult_credential_for_authorized_principal() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let authorized_principal = Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap();
    let credential = construct_adult_credential();
    api::add_adult(&env, issuer_id, authorized_principal, credential).expect("API call failed");
    let response = api::prepare_credential(
        &env,
        issuer_id,
        authorized_principal,
        &PrepareCredentialRequest {
            credential_spec: adult_credential_spec(),
            signed_id_alias: DUMMY_SIGNED_ID_ALIAS.clone(),
        },
    )
    .expect("API call failed");
    assert_matches!(response, Ok(_));
}
/// Verifies that different credentials are being created including II interactions.
#[test]
fn should_issue_credential_e2e() -> Result<(), CallError> {
    let env = env();
    let ii_id = install_canister(&env, II_WASM.clone());
    let issuer_id = install_issuer(
        &env,
        &IssuerInit {
            ic_root_key_der: env.root_key().to_vec(),
            idp_canister_ids: vec![ii_id],
            derivation_origin: DUMMY_DERIVATION_ORIGIN.to_string(),
            frontend_hostname: DUMMY_FRONTEND_HOSTNAME.to_string(),
        },
    );
    let identity_number = flows::register_anchor(&env, ii_id);
    let relying_party = FrontendHostname::from("https://some-dapp.com");
    let issuer = FrontendHostname::from("https://some-issuer.com");

    let prepare_id_alias_req = PrepareIdAliasRequest {
        identity_number,
        relying_party: relying_party.clone(),
        issuer: issuer.clone(),
    };

    let prepared_id_alias =
        ii_api::prepare_id_alias(&env, ii_id, principal_1(), prepare_id_alias_req)?
            .expect("prepare id_alias failed");

    let canister_sig_pk =
        CanisterSigPublicKey::try_from(prepared_id_alias.canister_sig_pk_der.as_ref())
            .expect("failed parsing canister sig pk");

    let get_id_alias_req = GetIdAliasRequest {
        identity_number,
        relying_party,
        issuer,
        rp_id_alias_jwt: prepared_id_alias.rp_id_alias_jwt,
        issuer_id_alias_jwt: prepared_id_alias.issuer_id_alias_jwt,
    };
    let id_alias_credentials = ii_api::get_id_alias(&env, ii_id, principal_1(), get_id_alias_req)?
        .expect("get id_alias failed");

    let root_pk_raw =
        extract_raw_root_pk_from_der(&env.root_key()).expect("Failed decoding IC root key.");
    let alias_tuple = get_verified_id_alias_from_jws(
        &id_alias_credentials
            .issuer_id_alias_credential
            .credential_jws,
        &id_alias_credentials.issuer_id_alias_credential.id_dapp,
        &canister_sig_pk.canister_id,
        &root_pk_raw,
        env.time().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
    )
    .expect("Invalid ID alias");

    api::add_adult(&env, issuer_id, alias_tuple.id_dapp, construct_adult_credential())?;

    for credential_spec in [
        adult_credential_spec(),
    ] {
        let prepared_credential = api::prepare_credential(
            &env,
            issuer_id,
            id_alias_credentials.issuer_id_alias_credential.id_dapp,
            &PrepareCredentialRequest {
                credential_spec: credential_spec.clone(),
                signed_id_alias: SignedIssuerIdAlias {
                    credential_jws: id_alias_credentials
                        .issuer_id_alias_credential
                        .credential_jws
                        .clone(),
                },
            },
        )?
        .expect("failed to prepare credential");

        let get_credential_response = api::get_credential(
            &env,
            issuer_id,
            id_alias_credentials.issuer_id_alias_credential.id_dapp,
            &GetCredentialRequest {
                credential_spec: credential_spec.clone(),
                signed_id_alias: SignedIssuerIdAlias {
                    credential_jws: id_alias_credentials
                        .issuer_id_alias_credential
                        .credential_jws
                        .clone(),
                },
                prepared_context: prepared_credential.prepared_context,
            },
        )?;
        let claims = verify_credential_jws_with_canister_id(
            &get_credential_response.unwrap().vc_jws,
            &issuer_id,
            &root_pk_raw,
            env.time().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        )
        .expect("credential verification failed");
        let vc_claims = claims.vc().expect("missing VC claims");
        println!("{:?}", vc_claims);

        // validate_claims_match_spec(vc_claims, &credential_spec).expect("Clam validation failed");

    }

    Ok(())
}

#[test]
fn should_configure() {
    let env = env();
    let issuer_id = install_canister(&env, CIVIV_CANISTER_BACKEND_WASM.clone());
    api::configure(&env, issuer_id, &DUMMY_ISSUER_INIT).expect("API call failed");
}


ic_cdk::export_candid!();


// Helper functions

fn construct_adult_credential () -> StoredCredential {
    let mut claim_map = HashMap::<String, ClaimValue>::new();
    claim_map.insert("Is over 18".to_string(), ClaimValue::Boolean(true));
       StoredCredential {
        id: "http://example.edu/credentials/3732".to_string(),
        type_: vec!["VerifiableCredential".to_string(), "VerifiedAdult".to_string()],
        context: vec![
            "https://www.w3.org/2018/credentials/v1".to_string(),
            "https://www.w3.org/2018/credentials/examples/v1".to_string(),
        ],
        issuer: "https://civic.com".to_string(),
        claim: vec![Claim{claims: claim_map}],
    }
}


// ==================================================


use std::collections::{ BTreeMap};
use identity_credential::credential::{CredentialBuilder, Subject};
use serde::{Serialize};
pub use serde_json::Value;
// use candid::CandidType;
use identity_core::common::Url;
use std::iter::repeat;

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum ClaimValue {
    Boolean(bool),
    Date(String),
    Text(String),
    Number(i64),
    Claim(Claim),
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct Claim {
    pub claims:HashMap<String, ClaimValue>,
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
            },
        }
    }
}


impl Claim {
    pub fn into(self) -> Subject {
        let btree_map: BTreeMap<String, Value> = self.claims.into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect();
        Subject::with_properties(btree_map) 
    }
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct StoredCredential {
    pub id: String, 
    pub type_: Vec<String>,
    pub context: Vec<String>,
    pub issuer: String,
    pub claim: Vec<Claim>,
}
#[derive(CandidType)]
pub enum CredentialError {
    NoCredentialsFound(String),
}


// Helper functions for constructing the credential that is returned from the canister 

/// Build a credentialSubject {
/// id: SubjectId, 
/// otherData
///  }
pub fn build_claims_into_credentialSubjects(claims: Vec<Claim>, subject: String) -> Vec<Subject> {
    claims.into_iter().zip(repeat(subject)).map(|(c, id )|{
        let mut sub = c.into();
        sub.id = Url::parse(id).ok();
        sub
    }).collect()
}


pub fn add_context(mut credential: CredentialBuilder, context: Vec<String>) -> CredentialBuilder {
    for c in context {
     credential = credential.context(Url::parse(c).unwrap());
    }
    credential
}

// pub fn add_types(mut credential: CredentialBuilder, types: Vec<String>) -> CredentialBuilder {
//     for t in types {
//      credential = credential.type_(t);
//     }
//     credential
// }