//! Handles consent messages that are displayed to the user when they are asked to consent to the sharing of a VC by the Civic Canister.
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use candid::candid_method;
use ic_cdk_macros::update;
use lazy_static::lazy_static;
use vc_util::issuer_api::{
    Icrc21ConsentInfo,Icrc21VcConsentMessageRequest,  Icrc21ConsentPreferences, Icrc21Error, Icrc21ErrorInfo,
};
use SupportedLanguage::{English, German};

/// Consent messages for the CivicPass VC to be shown and approved to the user during the VC sharing flow 
const VC_DESCRIPTION_EN: &str = r###"# Verifiable Credential

Credential that states that the holder possesses a Verifiable Credential."###;
const VC_DESCRIPTION_DE: &str = r###"# Verifiable Credential

Bescheinigung, aus der hervorgeht, dass der Inhaber einen Verifiable Credential besitzt."###;

lazy_static! {
    static ref CONSENT_MESSAGE_TEMPLATES: HashMap<(CredentialTemplateType, SupportedLanguage), &'static str> =
        HashMap::from([
            (
                (CredentialTemplateType::Credential, English),
                VC_DESCRIPTION_EN
            ),
            (
                (CredentialTemplateType::Credential, German),
                VC_DESCRIPTION_DE
            )
        ]);
}

/// Supported consent message types
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum CredentialTemplateType {
    Credential,
}

/// Supported languages for consent messages
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum SupportedLanguage {
    English,
    German,
}

impl From<Icrc21ConsentPreferences> for SupportedLanguage {
    fn from(value: Icrc21ConsentPreferences) -> Self {
        match &value.language.to_lowercase()[..2] {
            "de" => German,
            _ => English, // english is also the fallback
        }
    }
}

impl Display for SupportedLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            English => write!(f, "en"),
            German => write!(f, "de"),
        }
    }
}


/// Get the consent message for the given credential spec to be used during the VC sharing flow
#[update]
#[candid_method]
async fn vc_consent_message(
    req: Icrc21VcConsentMessageRequest,
) -> Result<Icrc21ConsentInfo, Icrc21Error> {
    get_vc_consent_message(
        &SupportedLanguage::from(req.preferences),
    )
}

/// Retrieve the consent message for the given credential type and language.
fn get_vc_consent_message(
    language: &SupportedLanguage,
) -> Result<Icrc21ConsentInfo, Icrc21Error> {
    render_consent_message(language).map(|message| Icrc21ConsentInfo {
        consent_message: message,
        language: format!("{}", language),
    })
}

/// Show the consent message with any arguments 
fn render_consent_message(
    language: &SupportedLanguage,
) -> Result<String, Icrc21Error> {
    let template = CONSENT_MESSAGE_TEMPLATES
        .get(&(
            CredentialTemplateType::Credential,
            language.clone(),
        ))
        .ok_or(Icrc21Error::ConsentMessageUnavailable(Icrc21ErrorInfo {
            description: "Consent message template not found".to_string(),
        }))?;
        Ok(template.to_string())   
}