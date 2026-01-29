//! Authentication parameters types for the different [`AuthType`]s.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

mod params_serde;

/// Parameters for the terms of service flow.
///
/// This type is only valid during account registration.
///
/// See [the spec] for how to use this.
///
/// [the spec]: https://spec.matrix.org/latest/client-server-api/#terms-of-service-at-registration
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct LoginTermsParams {
    /// A map from policy ID to the current definition of this policy document.
    pub policies: BTreeMap<String, PolicyDefinition>,
}

impl LoginTermsParams {
    /// Construct a new `LoginTermsParams` with the given policy documents.
    pub fn new(policies: BTreeMap<String, PolicyDefinition>) -> Self {
        Self { policies }
    }
}

/// The definition of a policy document.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PolicyDefinition {
    /// The version of this policy document.
    pub version: String,

    /// Map from language codes to details of the document in that language.
    ///
    /// Language codes SHOULD be formatted as per Section 2.2 of RFC 5646, though some
    /// implementations may use an underscore instead of dash (for example, en_US instead of
    /// en-US).
    #[serde(flatten)]
    pub translations: BTreeMap<String, PolicyTranslation>,
}

impl PolicyDefinition {
    /// Construct a new `PolicyDefinition` with the given version and translations.
    pub fn new(version: String, translations: BTreeMap<String, PolicyTranslation>) -> Self {
        Self { version, translations }
    }
}

/// Details about a translation of a policy document.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PolicyTranslation {
    /// The name of this document, in the appropriate language.
    pub name: String,

    /// A link to the text of this document, in the appropriate language.
    ///
    /// MUST be a valid URI with scheme `https://` or `http://`. Insecure HTTP is discouraged..
    pub url: String,
}

impl PolicyTranslation {
    /// Construct a new `PolicyTranslation` with the given name and URL.
    pub fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

/// Parameters for an [OAuth 2.0-based] UIAA flow.
///
/// [OAuth 2.0-based]: https://spec.matrix.org/latest/client-server-api/#oauth-authentication
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct OAuthParams {
    /// A URL pointing to the homeserver’s OAuth 2.0 account management web UI where the user can
    /// approve the action.
    ///
    /// Must be a valid URI with scheme `http://` or `https://`, the latter being recommended.
    pub url: String,
}

impl OAuthParams {
    /// Construct an `OAuthParams` with the given URL.
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::{LoginTermsParams, PolicyDefinition, PolicyTranslation};

    #[test]
    fn serialize_login_terms_params() {
        let privacy_definition = PolicyDefinition::new(
            "1".to_owned(),
            [
                (
                    "en-US".to_owned(),
                    PolicyTranslation::new(
                        "Privacy Policy".to_owned(),
                        "http://matrix.local/en-US/privacy".to_owned(),
                    ),
                ),
                (
                    "fr-FR".to_owned(),
                    PolicyTranslation::new(
                        "Politique de confidentialité".to_owned(),
                        "http://matrix.local/fr-FR/privacy".to_owned(),
                    ),
                ),
            ]
            .into(),
        );
        let params = LoginTermsParams::new([("privacy".to_owned(), privacy_definition)].into());

        assert_to_canonical_json_eq!(
            params,
            json!({
                "policies": {
                    "privacy": {
                        "en-US": {
                            "name": "Privacy Policy",
                            "url": "http://matrix.local/en-US/privacy",
                        },
                        "fr-FR": {
                            "name": "Politique de confidentialité",
                            "url": "http://matrix.local/fr-FR/privacy",
                        },
                        "version": "1",
                    },
                },
            })
        );
    }

    #[test]
    fn deserialize_login_terms_params() {
        // Missing version field in policy.
        let json = json!({
            "policies": {
                "privacy": {
                    "en-US": {
                        "name": "Privacy Policy",
                        "url": "http://matrix.local/en-US/privacy",
                    },
                    "fr-FR": {
                        "name": "Politique de confidentialité",
                        "url": "http://matrix.local/fr-FR/privacy",
                    },
                },
            },
        });

        from_json_value::<LoginTermsParams>(json).unwrap_err();

        // Valid params.
        let json = json!({
            "policies": {
                "privacy_policy": {
                    "en": {
                        "name": "Privacy Policy",
                        "url": "https://example.org/somewhere/privacy-1.2-en.html"
                    },
                    "fr": {
                        "name": "Politique de confidentialité",
                        "url": "https://example.org/somewhere/privacy-1.2-fr.html"
                    },
                    // Unsupported field will be ignored.
                    "foo": "bar",
                    "version": "1.2",
                },
                // No translations is fine.
                "terms_of_service": {
                    "version": "1.2",
                }
            }
        });

        let params = from_json_value::<LoginTermsParams>(json).unwrap();

        assert_eq!(params.policies.len(), 2);

        let policy = params.policies.get("privacy_policy").unwrap();
        assert_eq!(policy.version, "1.2");
        assert_eq!(policy.translations.len(), 2);
        let translation = policy.translations.get("en").unwrap();
        assert_eq!(translation.name, "Privacy Policy");
        assert_eq!(translation.url, "https://example.org/somewhere/privacy-1.2-en.html");
        let translation = policy.translations.get("fr").unwrap();
        assert_eq!(translation.name, "Politique de confidentialité");
        assert_eq!(translation.url, "https://example.org/somewhere/privacy-1.2-fr.html");

        let policy = params.policies.get("terms_of_service").unwrap();
        assert_eq!(policy.version, "1.2");
        assert_eq!(policy.translations.len(), 0);
    }
}
