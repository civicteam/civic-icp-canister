use std::collections::BTreeMap;
use identity_credential::credential::Subject;
use serde::{Serialize, Deserialize};
use candid::CandidType;

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum ClaimValue {
    Boolean(bool),
    Date(String),
    Text(String),
    Number(i64),
    Nested(Claim),
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub struct Claim {
claims:BTreeMap<String, ClaimValue>,
}
// #[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
// struct CredentialSubject {
//     id: String, 
//     claims: Vec<Claim>
// }

impl Claim {
    pub fn into(self) -> Subject {
        Subject::with_properties(self.claims)
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

pub fn build_claims_into_credentialSubjects(claims: Vec<Claim>) -> Vec<Subject> {
    claims.into_iter().map(|claim| claim.into()).collect()
}

