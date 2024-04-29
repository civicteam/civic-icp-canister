use std::collections::{HashMap, BTreeMap};
use identity_credential::credential::{CredentialBuilder, Subject};
use serde::{Serialize, Deserialize};
pub use serde_json::Value;
use candid::CandidType;
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