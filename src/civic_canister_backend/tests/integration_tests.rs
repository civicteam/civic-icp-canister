//! Tests related to issue_credential canister call.
extern crate civic_canister_backend;

use assert_matches::assert_matches;
use candid::Principal;
use canister_sig_util::{extract_raw_root_pk_from_der, CanisterSigPublicKey};
use canister_tests::api::internet_identity::vc_mvp as ii_api;
use canister_tests::flows;
use canister_tests::framework::{
    env, get_wasm_path, principal_1, principal_2, test_principal, II_WASM,
};
use civic_canister_backend::config::IssuerInit;
use civic_canister_backend::credential::{Claim, ClaimValue, CredentialError, FullCredential};
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
use std::time::UNIX_EPOCH;
use vc_util::issuer_api::{
    ArgumentValue, CredentialSpec, DerivationOriginData, DerivationOriginError,
    DerivationOriginRequest, GetCredentialRequest, Icrc21ConsentInfo, Icrc21ConsentPreferences,
    Icrc21Error, Icrc21VcConsentMessageRequest, IssueCredentialError, IssuedCredentialData,
    PrepareCredentialRequest, PreparedCredentialData, SignedIdAlias as SignedIssuerIdAlias,
};
use vc_util::{get_verified_id_alias_from_jws, verify_credential_jws_with_canister_id};

const DUMMY_ROOT_KEY: &str ="308182301d060d2b0601040182dc7c0503010201060c2b0601040182dc7c05030201036100adf65638a53056b2222c91bb2457b0274bca95198a5acbdadfe7fd72178f069bdea8d99e9479d8087a2686fc81bf3c4b11fe275570d481f1698f79d468afe0e57acc1e298f8b69798da7a891bbec197093ec5f475909923d48bfed6843dbed1f";
const DUMMY_II_CANISTER_ID: &str = "rwlgt-iiaaa-aaaaa-aaaaa-cai";
const DUMMY_DERIVATION_ORIGIN: &str = "https://y2aaj-miaaa-aaaad-aacxq-cai.ic0.app";
const DUMMY_FRONTEND_HOSTNAME: &str = "https://y2aaj-miaaa-aaaad-aacxq-cai.ic0.app";
const ISSUER_PRINCIPAL: &str = "tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae";

/// Dummy alias JWS for testing, valid wrt DUMMY_ROOT_KEY and DUMMY_II_CANISTER_ID.
/// id dapp: nugva-s7c6v-4yszt-koycv-5b623-an7q6-ha2nz-kz6rs-hawgl-nznbe-rqe
/// id alias: jkk22-zqdxc-kgpez-6sv2m-5pby4-wi4t2-prmoq-gf2ih-i2qtc-v37ac-5ae
const DUMMY_ALIAS_JWS: &str ="eyJqd2siOnsia3R5Ijoib2N0IiwiYWxnIjoiSWNDcyIsImsiOiJNRHd3REFZS0t3WUJCQUdEdUVNQkFnTXNBQW9BQUFBQUFBQUFBQUVCMGd6TTVJeXFMYUhyMDhtQTRWd2J5SmRxQTFyRVFUX2xNQnVVbmN5UDVVYyJ9LCJraWQiOiJkaWQ6aWNwOnJ3bGd0LWlpYWFhLWFhYWFhLWFhYWFhLWNhaSIsImFsZyI6IkljQ3MifQ.eyJleHAiOjE2MjAzMjk1MzAsImlzcyI6Imh0dHBzOi8vaWRlbnRpdHkuaWMwLmFwcC8iLCJuYmYiOjE2MjAzMjg2MzAsImp0aSI6ImRhdGE6dGV4dC9wbGFpbjtjaGFyc2V0PVVURi04LHRpbWVzdGFtcF9uczoxNjIwMzI4NjMwMDAwMDAwMDAwLGFsaWFzX2hhc2g6YTI3YzU4NTQ0MmUwN2RkZWFkZTRjNWE0YTAzMjdkMzA4NTE5NDAzYzRlYTM3NDIxNzBhZTRkYzk1YjIyZTQ3MyIsInN1YiI6ImRpZDppY3A6bnVndmEtczdjNnYtNHlzenQta295Y3YtNWI2MjMtYW43cTYtaGEybnota3o2cnMtaGF3Z2wtbnpuYmUtcnFlIiwidmMiOnsiQGNvbnRleHQiOiJodHRwczovL3d3dy53My5vcmcvMjAxOC9jcmVkZW50aWFscy92MSIsInR5cGUiOlsiVmVyaWZpYWJsZUNyZWRlbnRpYWwiLCJJbnRlcm5ldElkZW50aXR5SWRBbGlhcyJdLCJjcmVkZW50aWFsU3ViamVjdCI6eyJJbnRlcm5ldElkZW50aXR5SWRBbGlhcyI6eyJoYXNJZEFsaWFzIjoiamtrMjItenFkeGMta2dwZXotNnN2Mm0tNXBieTQtd2k0dDItcHJtb3EtZ2YyaWgtaTJxdGMtdjM3YWMtNWFlIn19fX0.2dn3omtjZXJ0aWZpY2F0ZVkBsdnZ96JkdHJlZYMBgwGDAYMCSGNhbmlzdGVygwGDAkoAAAAAAAAAAAEBgwGDAYMBgwJOY2VydGlmaWVkX2RhdGGCA1ggvlJBTZDgK1_9Vb3-18dWKIfy28WTjZ1YqdjFWWAIX96CBFgg0sz_P8xdqTDewOhKJUHmWFFrS7FQHnDotBDmmGoFfWCCBFgg_KZ0TVqubo_EGWoMUPA35BYZ4B5ZRkR_zDfNIQCwa46CBFggj_ZV-7o59iVEjztzZtpNnO9YC7GjbKmg2eDtJzGz1weCBFggXAzCWvb9h4qsVs41IUJBABzjSqAZ8DIzF_ghGHpGmHGCBFggJhbsbvKYt7rjLK5SI0NDc600o-ajSYQNuOXps6qUrdiCBFggBFQwZetJeY_gx6TQohTqUOskblddajS20DA0esxWoyWDAYIEWCA1U_ZYHVOz3Sdkb2HIsNoLDDiBuFfG3DxH6miIwRPra4MCRHRpbWWCA0mAuK7U3YmkvhZpc2lnbmF0dXJlWDC5cq4UxYy7cnkcw6yv5SCh4POY9u0iHecZuxO8E9oxIqXRdHmnYVF0Fv_R-aws0EBkdHJlZYMBggRYIOGnlc_3yXPTVrEJ1p3dKX5HxkMOziUnpA1HeXiQW4O8gwJDc2lngwJYIIOQR7wl3Ws9Jb8VP4rhIb37XKLMkkZ2P7WaZ5we60WGgwGCBFgg21-OewBgqt_-0AtHHHS4yPyQK9g6JTHaGUuSIw4QYgqDAlgg5bQnHHvS3FfM_BaiSL6n19qoXkuA1KoLWk963fOUMW-CA0A";
const DUMMY_ALIAS_ID_DAPP_PRINCIPAL: &str =
    "nugva-s7c6v-4yszt-koycv-5b623-an7q6-ha2nz-kz6rs-hawgl-nznbe-rqe";

lazy_static! {
    pub static ref CIVIV_CANISTER_BACKEND_WASM: Vec<u8> = {
        let def_path = PathBuf::from("../../")
            .join("target/wasm32-unknown-unknown/release/civic_canister_backend.wasm");
        let err = format!(
            "
        Could not find VC Issuer Wasm module for current build.
        I will look for it at {:?} (note that I run from {:?}).
        ",
            &def_path,
            &std::env::current_dir()
                .map(|x| x.display().to_string())
                .unwrap_or_else(|_| "an unknown directory".to_string())
        );
        get_wasm_path("CIVIV_CANISTER_BACKEND_WASM".to_string(), &def_path).expect(&err)
    };
    pub static ref DUMMY_ISSUER_INIT: IssuerInit = IssuerInit {
        ic_root_key_der: hex::decode(DUMMY_ROOT_KEY).unwrap(),
        idp_canister_ids: vec![Principal::from_text(DUMMY_II_CANISTER_ID).unwrap()],
        derivation_origin: DUMMY_DERIVATION_ORIGIN.to_string(),
        frontend_hostname: DUMMY_FRONTEND_HOSTNAME.to_string(),
        admin: Principal::from_text(ISSUER_PRINCIPAL).unwrap(),
        authorized_issuers: vec![Principal::from_text(ISSUER_PRINCIPAL).unwrap()],
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

    pub fn add_credentials(
        env: &StateMachine,
        canister_id: CanisterId,
        user: Principal,
        new_credentials: Vec<FullCredential>,
    ) -> Result<Result<String, CredentialError>, CallError> {
        let civic_issuer =
            Principal::from_text("tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae")
                .unwrap();
        call_candid_as(
            env,
            canister_id,
            civic_issuer,
            "add_credentials",
            (user, new_credentials),
        )
        .map(|(x,)| x)
    }

    pub fn add_credentials_with_sender(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        user: Principal,
        new_credentials: Vec<FullCredential>,
    ) -> Result<Result<String, CredentialError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "add_credentials",
            (user, new_credentials),
        )
        .map(|(x,)| x)
    }

    pub fn update_credential(
        env: &StateMachine,
        canister_id: CanisterId,
        sender: Principal,
        user: Principal,
        credential_id: String,
        updated_credential: FullCredential,
    ) -> Result<Result<String, CredentialError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "update_credential",
            (user, credential_id, updated_credential),
        )
        .map(|(x,)| x)
    }

    pub fn get_all_credentials(
        env: &StateMachine,
        canister_id: CanisterId,
        user: Principal,
    ) -> Result<Result<Vec<FullCredential>, CredentialError>, CallError> {
        call_candid(env, canister_id, "get_all_credentials", (user,)).map(|(x,)| x)
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

    pub fn remove_credential(
        env: &StateMachine,
        sender: Principal,
        canister_id: CanisterId,
        user: Principal,
        credential_id: String,
    ) -> Result<Result<String, CredentialError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            sender,
            "remove_credential",
            (user, credential_id),
        )
        .map(|(x,)| x)
    }

    pub fn add_issuer(
        env: &StateMachine,
        canister_id: CanisterId,
        authorized_principal: Principal,
        new_issuer: Principal,
    ) -> Result<Result<(), CredentialError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            authorized_principal,
            "add_issuer",
            (new_issuer,),
        )
        .map(|(x,)| x)
    }

    pub fn remove_issuer(
        env: &StateMachine,
        canister_id: CanisterId,
        authorized_principal: Principal,
        issuer: Principal,
    ) -> Result<Result<(), CredentialError>, CallError> {
        call_candid_as(
            env,
            canister_id,
            authorized_principal,
            "remove_issuer",
            (issuer,),
        )
        .map(|(x,)| x)
    }
}

fn adult_credential_spec() -> CredentialSpec {
    CredentialSpec {
        credential_type: "VerifiedAdult".to_string(),
        arguments: None,
    }
}

fn construct_adult_credential() -> FullCredential {
    let mut claim_map = HashMap::<String, ClaimValue>::new();
    claim_map.insert("Is over 18".to_string(), ClaimValue::Boolean(true));
    FullCredential {
        id: "http://example.edu/credentials/3732".to_string(),
        type_: vec![
            "VerifiableCredential".to_string(),
            "VerifiedAdult".to_string(),
        ],
        context: vec![
            "https://www.w3.org/2018/credentials/v1".to_string(),
            "https://www.w3.org/2018/credentials/examples/v1".to_string(),
        ],
        issuer: "https://civic.com".to_string(),
        claim: vec![Claim { claims: claim_map }],
    }
}

/// Test: VC consent message for adult VC
#[test]
fn should_return_vc_consent_message_for_adult_vc() {
    let test_cases = [
        ("en-US", "en", "# Verified Adult"),
        ("de-DE", "de", "# Erwachsene Person"),
        ("ja-JP", "en", "# Verified Adult"), // test fallback language
    ];
    let env = env();
    let canister_id = install_canister(&env, CIVIV_CANISTER_BACKEND_WASM.clone());

    for (requested_language, actual_language, consent_message_snippet) in test_cases {
        let mut args = HashMap::new();
        args.insert("minAge".to_string(), ArgumentValue::Int(18));
        let consent_message_request = Icrc21VcConsentMessageRequest {
            credential_spec: CredentialSpec {
                credential_type: "VerifiedAdult".to_string(),
                arguments: Some(args),
            },
            preferences: Icrc21ConsentPreferences {
                language: requested_language.to_string(),
            },
        };

        let response =
            api::vc_consent_message(&env, canister_id, principal_1(), &consent_message_request)
                .expect("API call failed")
                .expect("Consent message error");
        assert_eq!(response.language, actual_language);
        assert!(response
            .consent_message
            .starts_with(consent_message_snippet));
    }
}

/// Test: Derivation origin
#[test]
fn should_return_derivation_origin() {
    let env = env();
    let canister_id = install_canister(&env, CIVIV_CANISTER_BACKEND_WASM.clone());
    let frontend_hostname = format!("https://{}.icp0.io", canister_id.to_text());
    let req = DerivationOriginRequest { frontend_hostname };
    let response = api::derivation_origin(&env, canister_id, principal_1(), &req)
        .expect("API call failed")
        .expect("derivation_origin error");
    assert_eq!(response.origin, req.frontend_hostname);
}

/// Test: Derivation origin with custom init
#[test]
fn should_return_derivation_origin_with_custom_init() {
    let env = env();
    let custom_init = IssuerInit {
        ic_root_key_der: hex::decode(DUMMY_ROOT_KEY).unwrap(),
        idp_canister_ids: vec![Principal::from_text(DUMMY_II_CANISTER_ID).unwrap()],
        derivation_origin: "https://derivation_origin".to_string(),
        frontend_hostname: "https://frontend.host.name".to_string(),
        admin: Principal::from_text(ISSUER_PRINCIPAL).unwrap(),
        authorized_issuers: vec![Principal::from_text(ISSUER_PRINCIPAL).unwrap()],
    };
    let issuer_id = install_issuer(&env, &custom_init);
    let response = api::derivation_origin(
        &env,
        issuer_id,
        principal_1(),
        &DerivationOriginRequest {
            frontend_hostname: custom_init.frontend_hostname.clone(),
        },
    )
    .expect("API call failed")
    .expect("derivation_origin error");
    assert_eq!(response.origin, custom_init.derivation_origin);
}

/// Test that adding and retrieving a credential will return the same credential data, the fields are compressed and then converted back
#[test]
fn should_return_same_credential_data_after_internal_compression() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let principal = principal_1();
    let credential1 = construct_adult_credential();
    let mut credential2 = construct_adult_credential();
    credential2.issuer = "other-issuer".to_string();
    api::add_credentials(&env, issuer_id, principal, vec![credential1.clone()])
        .expect("failed to add credential");
    api::add_credentials(&env, issuer_id, principal, vec![credential2.clone()])
        .expect("failed to add credential");
    let response = api::get_all_credentials(&env, issuer_id, principal)
        .expect("API call failed")
        .expect("get_all_credentials error");
    assert_eq!(response[0].issuer, credential1.issuer);
    assert_eq!(response[1].issuer, credential2.issuer);
}

/// Test that updating an url field is handled correctly 
#[test]
fn should_handle_the_update_of_url_fields_inside_internal_compression() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let principal = principal_1();
    let original_credential = construct_adult_credential();
    let mut updated_credential = construct_adult_credential();
    updated_credential.issuer = "updated-issuer".to_string();
    let id = original_credential.id.clone();

    let civic_issuer =
        Principal::from_text("tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae")
            .unwrap();


    api::add_credentials(&env, issuer_id, principal, vec![original_credential])
        .expect("failed to add credential");
    api::update_credential(&env, issuer_id, civic_issuer, principal, id, updated_credential.clone())
        .expect("failed to update credential");
    let response = api::get_all_credentials(&env, issuer_id, principal)
        .expect("API call failed")
        .expect("get_all_credentials error");
    assert_eq!(response[0].issuer, updated_credential.issuer);
}



#[test]
fn should_update_credential_successfully() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let principal = principal_1();
    let original_credential = construct_adult_credential();
    let mut updated_credential = construct_adult_credential();
    updated_credential.claim[0]
        .claims
        .entry("Is over 18".to_string())
        .and_modify(|x| *x = ClaimValue::Boolean(false));
    let id = original_credential.id.clone();

    // Add a credential first to update it later
    let _ = api::add_credentials(&env, issuer_id, principal, vec![original_credential])
        .expect("failed to add credential");

    let civic_issuer =
        Principal::from_text("tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae")
            .unwrap();

    // Update the credential
    let response = api::update_credential(
        &env,
        issuer_id,
        civic_issuer,
        principal,
        id.clone(),
        updated_credential,
    )
    .expect("API call failed");

    assert_matches!(response, Ok(_));

    let stored_updated_credential = api::get_all_credentials(&env, issuer_id, principal)
        .expect("API call failed")
        .expect("get_all_credentials error");
    // assert there is only one version of the VC
    assert_eq!(stored_updated_credential.len(), 1);
    // that was changed to the updated_credential
    assert_eq!(stored_updated_credential[0].id, id);
    assert_matches!(
        stored_updated_credential[0].claim[0]
            .claims
            .get("Is over 18")
            .unwrap(),
        &ClaimValue::Boolean(false)
    );
}


/// Test: Update credential failure if not found
#[test]
fn should_fail_update_credential_if_not_found() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let principal = principal_1();
    let civic_issuer =
        Principal::from_text("tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae")
            .unwrap();
    let credential = construct_adult_credential();

    // Attempt to update a non-existing credential
    let response = api::update_credential(
        &env,
        issuer_id,
        civic_issuer,
        principal,
        "non-existing-id".to_string(),
        credential.clone(),
    )
    .expect("API call should fail");

    assert_matches!(response, Err(CredentialError::NoCredentialFound(_)));
}

/// Test: VC consent message failure if not supported
#[test]
fn should_fail_vc_consent_message_if_not_supported() {
    let env = env();
    let canister_id = install_canister(&env, CIVIV_CANISTER_BACKEND_WASM.clone());

    let consent_message_request = Icrc21VcConsentMessageRequest {
        credential_spec: CredentialSpec {
            credential_type: "VerifiedResident".to_string(),
            arguments: None,
        },
        preferences: Icrc21ConsentPreferences {
            language: "en-US".to_string(),
        },
    };

    let response =
        api::vc_consent_message(&env, canister_id, principal_1(), &consent_message_request)
            .expect("API call failed");
    assert_matches!(response, Err(Icrc21Error::UnsupportedCanisterCall(_)));
}

/// Test: Unauthorized credential removal
#[test]
fn should_not_remove_credentials_for_unauthorized_principal() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let principal: Principal = principal_1();
    let unauthorized_principal = principal_2();
    let credential = construct_adult_credential();

    // Add a credential first to attempt removing it later
    let _ = api::add_credentials(&env, issuer_id, principal, vec![credential.clone()])
        .expect("API call failed");

    // Attempt to remove the credential with an unauthorized principal
    let response = api::remove_credential(
        &env,
        unauthorized_principal,
        issuer_id,
        unauthorized_principal,
        credential.id,
    )
    .expect("API call failed");

    // Ensure the error is returned
    assert_matches!(response, Err(CredentialError::UnauthorizedSubject(_)));
}

/// Test: Prepare credential for wrong sender
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
        Err(IssueCredentialError::InvalidIdAlias(e)) if e.contains("Id alias could not be verified")
    );
}

/// Test: Get credential for wrong sender
#[test]
fn should_fail_get_credential_for_wrong_sender() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let signed_id_alias = DUMMY_SIGNED_ID_ALIAS.clone();
    let authorized_principal = Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap();
    let _ = api::add_credentials(
        &env,
        issuer_id,
        authorized_principal,
        vec![construct_adult_credential()],
    )
    .expect("failed to add employee");
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
    );

    match get_credential_response {
        Ok(Ok(_)) => panic!("Expected Err(IssueCredentialError::InvalidIdAlias(_)), got Ok"),
        Ok(Err(IssueCredentialError::InvalidIdAlias(e))) => {
            assert!(
                e.contains("Id alias could not be verified"),
                "Expected error message to contain 'id alias could not be verified', got: {}",
                e
            );
        }
        other => panic!(
            "Expected Err(IssueCredentialError::InvalidIdAlias(_)), got: {:?}",
            other
        ),
    }
}

/// Test: Prepare credential for anonymous caller
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

    match response {
        Err(IssueCredentialError::InvalidIdAlias(e)) => {
            assert!(
                e.contains("Id alias could not be verified"),
                "Expected error message to contain 'id alias could not be verified', got: {}",
                e
            );
        }
        _ => panic!(
            "Expected Err(IssueCredentialError::InvalidIdAlias(_)), got: {:?}",
            response
        ),
    }
}

/// Test: Prepare credential for wrong root key
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
            admin: Principal::from_text(ISSUER_PRINCIPAL).unwrap(),
            authorized_issuers: vec![Principal::from_text(ISSUER_PRINCIPAL).unwrap()],
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

/// Test: Prepare credential for wrong IDP canister ID
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
            admin: Principal::from_text(ISSUER_PRINCIPAL).unwrap(),
            authorized_issuers: vec![Principal::from_text(ISSUER_PRINCIPAL).unwrap()],
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

/// Test: Prepare adult credential for authorized principal
#[test]
fn should_prepare_adult_credential_for_authorized_principal() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let authorized_principal = Principal::from_text(DUMMY_ALIAS_ID_DAPP_PRINCIPAL).unwrap();
    let credential = construct_adult_credential();
    let _ = api::add_credentials(&env, issuer_id, authorized_principal, vec![credential])
        .expect("API call failed");
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

/// Test: Issue credential end-to-end
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
            admin: Principal::from_text(ISSUER_PRINCIPAL).unwrap(),
            authorized_issuers: vec![Principal::from_text(ISSUER_PRINCIPAL).unwrap()],
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

    let _ = api::add_credentials(
        &env,
        issuer_id,
        alias_tuple.id_dapp,
        vec![construct_adult_credential()],
    )?;

    for credential_spec in [adult_credential_spec()] {
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

/// Test: Add duplicate credentials
#[test]
fn should_not_add_duplicate_credentials() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let principal = principal_1();
    let credential = construct_adult_credential();

    // Add the credential for the first time
    let _ = api::add_credentials(&env, issuer_id, principal, vec![credential.clone()])
        .expect("API call failed");

    // Attempt to add the same credential again
    let response = api::add_credentials(&env, issuer_id, principal, vec![credential.clone()])
        .expect("API call failed")
        .unwrap(); // Unwrap the Result to access the inner String value

    println!("Response from second add: {}", response);
    assert!(response.contains("Added credentials"));

    // Ensure the duplicate is not added
    assert!(response.contains("Added credentials"));

    // Retrieve all stored credentials for the principal
    let stored_credentials = api::get_all_credentials(&env, issuer_id, principal)
        .expect("API call failed")
        .expect("get_all_credentials error");

    println!("Stored credentials: {:?}", stored_credentials);

    // Assert that only one credential is stored
    assert_eq!(
        stored_credentials.len(),
        1,
        "Expected only one credential, but found {}",
        stored_credentials.len()
    );
}

/// Test: Remove credential successfully
#[test]
fn should_remove_credential_successfully() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let principal = principal_1();
    let credential = construct_adult_credential();
    let civic_issuer =
        Principal::from_text("tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae")
            .unwrap();

    // Add the credential first
    let _ = api::add_credentials(&env, issuer_id, principal, vec![credential.clone()])
        .expect("API call failed");

    // Ensure the credential is added
    let stored_credentials = api::get_all_credentials(&env, issuer_id, principal)
        .expect("API call failed")
        .expect("get_all_credentials error");
    assert_eq!(stored_credentials.len(), 1);

    // Remove the credential
    let response = api::remove_credential(
        &env,
        civic_issuer,
        issuer_id,
        principal,
        credential.id.clone(),
    )
    .expect("API call failed")
    .expect("remove_credential error");

    assert!(response.contains("Credential removed successfully"));

    // Ensure the credential is removed
    let stored_credentials_after_removal = api::get_all_credentials(&env, issuer_id, principal)
        .expect("API call failed")
        .expect("get_all_credentials error");
    assert_eq!(stored_credentials_after_removal.len(), 0);
}

/// Test: Remove nonexistent credential
#[test]
fn should_fail_to_remove_nonexistent_credential() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let principal = principal_1();
    let civic_issuer =
        Principal::from_text("tglqb-kbqlj-to66e-3w5sg-kkz32-c6ffi-nsnta-vj2gf-vdcc5-5rzjk-jae")
            .unwrap();
    let credential_id = "nonexistent_id".to_string();

    // Attempt to remove a non-existing credential
    let response = api::remove_credential(&env, civic_issuer, issuer_id, principal, credential_id)
        .expect("API call failed");

    // Ensure the error is returned
    assert_matches!(response, Err(CredentialError::NoCredentialFound(_)));
}

/// Test: Add credentials for unauthorized principal
#[test]
fn should_not_add_credentials_for_unauthorized_principal() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let unauthorized_principal = principal_2(); // Use a different principal for unauthorized access
    let credential = construct_adult_credential();

    // Attempt to add credentials with an unauthorized principal
    let response = api::add_credentials_with_sender(
        &env,
        issuer_id,
        unauthorized_principal,
        unauthorized_principal,
        vec![credential],
    )
    .expect("API call failed");

    // Ensure the error is returned
    assert_matches!(response, Err(CredentialError::UnauthorizedSubject(_)));
}

/// Test: Update credentials for unauthorized principal
#[test]
fn should_not_update_credentials_for_unauthorized_principal() {
    let env = env();
    let issuer_id = install_issuer(&env, &DUMMY_ISSUER_INIT);
    let principal = principal_1();
    let unauthorized_principal = principal_2();
    let original_credential = construct_adult_credential();
    let updated_credential = construct_adult_credential();

    // Add a credential first to attempt updating it later
    let _ = api::add_credentials(
        &env,
        issuer_id,
        principal,
        vec![original_credential.clone()],
    )
    .expect("API call failed");

    // Attempt to update the credential with an unauthorized principal
    let response = api::update_credential(
        &env,
        issuer_id,
        unauthorized_principal,
        unauthorized_principal,
        original_credential.id,
        updated_credential,
    )
    .expect("API call failed");

    // Ensure the error is returned
    assert_matches!(response, Err(CredentialError::UnauthorizedSubject(_)));
}

/// Test: Configure canister
#[test]
fn should_configure() {
    let env = env();
    let issuer_id = install_canister(&env, CIVIV_CANISTER_BACKEND_WASM.clone());
    api::configure(&env, issuer_id, &DUMMY_ISSUER_INIT).expect("API call failed");
}

ic_cdk::export_candid!();
