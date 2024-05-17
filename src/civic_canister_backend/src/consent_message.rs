//! This module contains the various consent messages that is displayed to the user when they are asked to consent to the issuance of a credential.

use crate::credential::{SupportedCredentialType, verify_credential_spec};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use vc_util::issuer_api::{
    CredentialSpec, Icrc21ConsentInfo, Icrc21ConsentPreferences, Icrc21Error, Icrc21ErrorInfo,
};
use SupportedLanguage::{English, German};

/// Consent messages for the VerifiedAdult VC to be shown and approved to the user during the VC sharing flow 
const ADULT_VC_DESCRIPTION_EN: &str = r###"# Verified Adult

Credential that states that the holder's age is at least 18 years."###;
const ADULT_VC_DESCRIPTION_DE: &str = r###"# Erwachsene Person

Ausweis, der bestätigt, dass der Besitzer oder die Besitzerin mindestens 18 Jahre alt ist."###;

lazy_static! {
    static ref CONSENT_MESSAGE_TEMPLATES: HashMap<(CredentialTemplateType, SupportedLanguage), &'static str> =
        HashMap::from([
            (
                (CredentialTemplateType::VerifiedAdult, English),
                ADULT_VC_DESCRIPTION_EN
            ),
            (
                (CredentialTemplateType::VerifiedAdult, German),
                ADULT_VC_DESCRIPTION_DE
            )
        ]);
}

/// Supported consent message types
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum CredentialTemplateType {
    VerifiedAdult,
}

/// Supported languages for consent messages
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum SupportedLanguage {
    English,
    German,
}

impl From<&SupportedCredentialType> for CredentialTemplateType {
    fn from(value: &SupportedCredentialType) -> Self {
        match value {
            SupportedCredentialType::VerifiedAdult => CredentialTemplateType::VerifiedAdult,
        }
    }
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

/// Retrieve the consent message for the given credential type and language.
pub fn get_vc_consent_message(
    credential_spec: &CredentialSpec,
    language: &SupportedLanguage,
) -> Result<Icrc21ConsentInfo, Icrc21Error> {
    render_consent_message(credential_spec, language).map(|message| Icrc21ConsentInfo {
        consent_message: message,
        language: format!("{}", language),
    })
}

/// Show the consent message with any arguments 
fn render_consent_message(
    credential_spec: &CredentialSpec,
    language: &SupportedLanguage,
) -> Result<String, Icrc21Error> {
    let credential_type = match verify_credential_spec(credential_spec) {
        Ok(credential_type) => credential_type,
        Err(err) => {
            return Err(Icrc21Error::UnsupportedCanisterCall(Icrc21ErrorInfo {
                description: err,
            }));
        }
    };
    let template = CONSENT_MESSAGE_TEMPLATES
        .get(&(
            CredentialTemplateType::from(&credential_type),
            language.clone(),
        ))
        .ok_or(Icrc21Error::ConsentMessageUnavailable(Icrc21ErrorInfo {
            description: "Consent message template not found".to_string(),
        }))?;
        Ok(template.to_string())   
}