use std::collections::{HashMap, BTreeMap};
use identity_credential::credential::{CredentialBuilder, Subject};
use serde::{Serialize, Deserialize};
pub use serde_json::Value;
use candid::CandidType;
use identity_core::common::Url;
use std::iter::repeat;

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
    pub(crate)  claims:HashMap<String, ClaimValue>,
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
    pub(crate) fn into(self) -> Subject {
        let btree_map: BTreeMap<String, Value> = self.claims.into_iter()
        .map(|(k, v)| (k, v.into()))
        .collect();
        Subject::with_properties(btree_map) 
    }
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub(crate) struct StoredCredential {
    pub(crate)  id: String, 
    pub(crate)  type_: Vec<String>,
    pub(crate)  context: Vec<String>,
    pub(crate)  issuer: String,
    pub(crate)  claim: Vec<Claim>,
}
#[derive(CandidType)]
pub(crate) enum CredentialError {
    NoCredentialsFound(String),
}

pub(crate) fn build_claims_into_credential_subjects(claims: Vec<Claim>, subject: String) -> Vec<Subject> {
    claims.into_iter().zip(repeat(subject)).map(|(c, id )|{
        let mut sub = c.into();
        sub.id = Url::parse(id).ok();
        sub
    }).collect()
}


pub(crate) fn add_context(mut credential: CredentialBuilder, context: Vec<String>) -> CredentialBuilder {
    for c in context {
     credential = credential.context(Url::parse(c).unwrap());
    }
    credential
}