use serde::Deserialize;
use candid::{CandidType, Principal};
use std::{cell::RefCell, collections::HashMap};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Credential {
    pub id: String,
    pub issuer: String,
    pub context: Vec<String>,
    pub claims: Vec<Claim>,
}

thread_local! {
    static CREDENTIALS: RefCell<HashMap<Principal, Vec<Credential>>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Claim {
    pub claim_type: String,
    pub value: String,
}

#[ic_cdk::update]
fn add_credential(principal: Principal, new_credential: Credential) -> String {
    CREDENTIALS.with(|store| {
        let mut store = store.borrow_mut();
        store.entry(principal).or_insert_with(Vec::new).push(new_credential.clone());
        format!("Credential added: {:?}", new_credential)
    })
}

#[ic_cdk::query]
fn get_credentials(principal: Principal) -> Vec<Credential> {
    CREDENTIALS.with(|store| {
        let store = store.borrow();
        store.get(&principal).cloned().unwrap_or_default()
    })
}

ic_cdk::export_candid!();
