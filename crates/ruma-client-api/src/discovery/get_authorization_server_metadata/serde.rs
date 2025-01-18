use std::collections::BTreeSet;

use serde::{de, Deserialize};
use url::Url;

use super::msc2965::{
    AccountManagementAction, AuthorizationServerMetadata, CodeChallengeMethod, GrantType, Prompt,
    ResponseMode, ResponseType,
};

impl<'de> Deserialize<'de> for AuthorizationServerMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = AuthorizationServerMetadataDeHelper::deserialize(deserializer)?;

        let AuthorizationServerMetadataDeHelper {
            issuer,
            authorization_endpoint,
            token_endpoint,
            registration_endpoint,
            response_types_supported,
            response_modes_supported,
            grant_types_supported,
            revocation_endpoint,
            code_challenge_methods_supported,
            account_management_uri,
            account_management_actions_supported,
            device_authorization_endpoint,
            prompt_values_supported,
        } = helper;

        // Require `code` in `response_types_supported`.
        if !response_types_supported.contains(&ResponseType::Code) {
            return Err(de::Error::custom("missing value `code` in `response_types_supported`"));
        }

        // Require `query` and `fragment` in `response_modes_supported`.
        if let Some(response_modes) = &response_modes_supported {
            let query_found = response_modes.contains(&ResponseMode::Query);
            let fragment_found = response_modes.contains(&ResponseMode::Fragment);

            if !query_found && !fragment_found {
                return Err(de::Error::custom(
                    "missing values `query` and `fragment` in `response_modes_supported`",
                ));
            }
            if !query_found {
                return Err(de::Error::custom(
                    "missing value `query` in `response_modes_supported`",
                ));
            }
            if !fragment_found {
                return Err(de::Error::custom(
                    "missing value `fragment` in `response_modes_supported`",
                ));
            }
        }
        // If the field is missing, the default value is `["query", "fragment"]`, according to
        // RFC8414.
        let response_modes_supported = response_modes_supported
            .unwrap_or_else(|| [ResponseMode::Query, ResponseMode::Fragment].into());

        // Require `authorization_code` and `refresh_token` in `grant_types_supported`.
        let authorization_code_found =
            grant_types_supported.contains(&GrantType::AuthorizationCode);
        let refresh_token_found = grant_types_supported.contains(&GrantType::RefreshToken);
        if !authorization_code_found && !refresh_token_found {
            return Err(de::Error::custom(
                    "missing values `authorization_code` and `refresh_token` in `grant_types_supported`",
                ));
        }
        if !authorization_code_found {
            return Err(de::Error::custom(
                "missing value `authorization_code` in `grant_types_supported`",
            ));
        }
        if !refresh_token_found {
            return Err(de::Error::custom(
                "missing value `refresh_token` in `grant_types_supported`",
            ));
        }

        // Require `S256` in `code_challenge_methods_supported`.
        if !code_challenge_methods_supported.contains(&CodeChallengeMethod::S256) {
            return Err(de::Error::custom(
                "missing value `s256` in `code_challenge_methods_supported`",
            ));
        }

        Ok(AuthorizationServerMetadata {
            issuer,
            authorization_endpoint,
            token_endpoint,
            registration_endpoint,
            response_types_supported,
            response_modes_supported,
            grant_types_supported,
            revocation_endpoint,
            code_challenge_methods_supported,
            account_management_uri,
            account_management_actions_supported,
            device_authorization_endpoint,
            prompt_values_supported,
        })
    }
}

#[derive(Deserialize)]
struct AuthorizationServerMetadataDeHelper {
    issuer: Url,
    authorization_endpoint: Url,
    token_endpoint: Url,
    registration_endpoint: Option<Url>,
    response_types_supported: BTreeSet<ResponseType>,
    response_modes_supported: Option<BTreeSet<ResponseMode>>,
    grant_types_supported: BTreeSet<GrantType>,
    revocation_endpoint: Url,
    code_challenge_methods_supported: BTreeSet<CodeChallengeMethod>,
    account_management_uri: Option<Url>,
    #[serde(default)]
    account_management_actions_supported: BTreeSet<AccountManagementAction>,
    device_authorization_endpoint: Option<Url>,
    #[serde(default)]
    prompt_values_supported: Vec<Prompt>,
}

#[cfg(test)]
mod tests {
    use as_variant::as_variant;
    use serde_json::{from_value as from_json_value, value::Map as JsonMap, Value as JsonValue};
    use url::Url;

    use crate::discovery::get_authorization_server_metadata::{
        msc2965::{
            AccountManagementAction, AuthorizationServerMetadata, CodeChallengeMethod, GrantType,
            ResponseMode, ResponseType,
        },
        tests::authorization_server_metadata_json,
    };

    /// A valid `AuthorizationServerMetadata` with all fields and values, as a JSON object.
    fn authorization_server_metadata_object() -> JsonMap<String, JsonValue> {
        as_variant!(authorization_server_metadata_json(), JsonValue::Object).unwrap()
    }

    /// Get a mutable reference to the array value with the given key in the given object.
    ///
    /// Panics if the property doesn't exist or is not an array.
    fn get_mut_array<'a>(
        object: &'a mut JsonMap<String, JsonValue>,
        key: &str,
    ) -> &'a mut Vec<JsonValue> {
        object.get_mut(key).unwrap().as_array_mut().unwrap()
    }

    #[test]
    fn metadata_all_fields() {
        let metadata_object = authorization_server_metadata_object();
        let metadata =
            from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap();

        assert_eq!(metadata.issuer.as_str(), "https://server.local/");
        assert_eq!(metadata.authorization_endpoint.as_str(), "https://server.local/authorize");
        assert_eq!(metadata.token_endpoint.as_str(), "https://server.local/token");
        assert_eq!(
            metadata.registration_endpoint.as_ref().map(Url::as_str),
            Some("https://server.local/register")
        );

        assert_eq!(metadata.response_types_supported.len(), 1);
        assert!(metadata.response_types_supported.contains(&ResponseType::Code));

        assert_eq!(metadata.response_modes_supported.len(), 2);
        assert!(metadata.response_modes_supported.contains(&ResponseMode::Query));
        assert!(metadata.response_modes_supported.contains(&ResponseMode::Fragment));

        assert_eq!(metadata.grant_types_supported.len(), 2);
        assert!(metadata.grant_types_supported.contains(&GrantType::AuthorizationCode));
        assert!(metadata.grant_types_supported.contains(&GrantType::RefreshToken));

        assert_eq!(metadata.revocation_endpoint.as_str(), "https://server.local/revoke");

        assert_eq!(metadata.code_challenge_methods_supported.len(), 1);
        assert!(metadata.code_challenge_methods_supported.contains(&CodeChallengeMethod::S256));

        assert_eq!(
            metadata.account_management_uri.as_ref().map(Url::as_str),
            Some("https://server.local/account")
        );
        assert_eq!(metadata.account_management_actions_supported.len(), 6);
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::Profile));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::SessionsList));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::SessionView));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::SessionEnd));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::AccountDeactivate));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::CrossSigningReset));

        assert_eq!(
            metadata.device_authorization_endpoint.as_ref().map(Url::as_str),
            Some("https://server.local/device")
        );
    }

    #[test]
    fn metadata_no_optional_fields() {
        let mut metadata_object = authorization_server_metadata_object();
        assert!(metadata_object.remove("registration_endpoint").is_some());
        assert!(metadata_object.remove("response_modes_supported").is_some());
        assert!(metadata_object.remove("account_management_uri").is_some());
        assert!(metadata_object.remove("account_management_actions_supported").is_some());
        assert!(metadata_object.remove("device_authorization_endpoint").is_some());

        let metadata =
            from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap();

        assert_eq!(metadata.issuer.as_str(), "https://server.local/");
        assert_eq!(metadata.authorization_endpoint.as_str(), "https://server.local/authorize");
        assert_eq!(metadata.token_endpoint.as_str(), "https://server.local/token");
        assert_eq!(metadata.registration_endpoint, None);

        assert_eq!(metadata.response_types_supported.len(), 1);
        assert!(metadata.response_types_supported.contains(&ResponseType::Code));

        assert_eq!(metadata.response_modes_supported.len(), 2);
        assert!(metadata.response_modes_supported.contains(&ResponseMode::Query));
        assert!(metadata.response_modes_supported.contains(&ResponseMode::Fragment));

        assert_eq!(metadata.grant_types_supported.len(), 2);
        assert!(metadata.grant_types_supported.contains(&GrantType::AuthorizationCode));
        assert!(metadata.grant_types_supported.contains(&GrantType::RefreshToken));

        assert_eq!(metadata.revocation_endpoint.as_str(), "https://server.local/revoke");

        assert_eq!(metadata.code_challenge_methods_supported.len(), 1);
        assert!(metadata.code_challenge_methods_supported.contains(&CodeChallengeMethod::S256));

        assert_eq!(metadata.account_management_uri, None);
        assert_eq!(metadata.account_management_actions_supported.len(), 0);

        assert_eq!(metadata.device_authorization_endpoint, None);
    }

    #[test]
    fn metadata_additional_values() {
        let mut metadata_object = authorization_server_metadata_object();
        get_mut_array(&mut metadata_object, "response_types_supported").push("custom".into());
        get_mut_array(&mut metadata_object, "response_modes_supported").push("custom".into());
        get_mut_array(&mut metadata_object, "grant_types_supported").push("custom".into());
        get_mut_array(&mut metadata_object, "code_challenge_methods_supported")
            .push("custom".into());
        get_mut_array(&mut metadata_object, "account_management_actions_supported")
            .push("custom".into());

        let metadata =
            from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap();

        assert_eq!(metadata.issuer.as_str(), "https://server.local/");
        assert_eq!(metadata.authorization_endpoint.as_str(), "https://server.local/authorize");
        assert_eq!(metadata.token_endpoint.as_str(), "https://server.local/token");
        assert_eq!(
            metadata.registration_endpoint.as_ref().map(Url::as_str),
            Some("https://server.local/register")
        );

        assert_eq!(metadata.response_types_supported.len(), 2);
        assert!(metadata.response_types_supported.contains(&ResponseType::Code));
        assert!(metadata.response_types_supported.contains(&ResponseType::from("custom")));

        assert_eq!(metadata.response_modes_supported.len(), 3);
        assert!(metadata.response_modes_supported.contains(&ResponseMode::Query));
        assert!(metadata.response_modes_supported.contains(&ResponseMode::Fragment));
        assert!(metadata.response_modes_supported.contains(&ResponseMode::from("custom")));

        assert_eq!(metadata.grant_types_supported.len(), 3);
        assert!(metadata.grant_types_supported.contains(&GrantType::AuthorizationCode));
        assert!(metadata.grant_types_supported.contains(&GrantType::RefreshToken));
        assert!(metadata.grant_types_supported.contains(&GrantType::from("custom")));

        assert_eq!(metadata.revocation_endpoint.as_str(), "https://server.local/revoke");

        assert_eq!(metadata.code_challenge_methods_supported.len(), 2);
        assert!(metadata.code_challenge_methods_supported.contains(&CodeChallengeMethod::S256));
        assert!(metadata
            .code_challenge_methods_supported
            .contains(&CodeChallengeMethod::from("custom")));

        assert_eq!(
            metadata.account_management_uri.as_ref().map(Url::as_str),
            Some("https://server.local/account")
        );
        assert_eq!(metadata.account_management_actions_supported.len(), 7);
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::Profile));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::SessionsList));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::SessionView));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::SessionEnd));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::AccountDeactivate));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::CrossSigningReset));
        assert!(metadata
            .account_management_actions_supported
            .contains(&AccountManagementAction::from("custom")));

        assert_eq!(
            metadata.device_authorization_endpoint.as_ref().map(Url::as_str),
            Some("https://server.local/device")
        );
    }

    #[test]
    fn metadata_missing_required_fields() {
        let original_metadata_object = authorization_server_metadata_object();

        let mut metadata_object = original_metadata_object.clone();
        assert!(metadata_object.remove("issuer").is_some());
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        assert!(metadata_object.remove("authorization_endpoint").is_some());
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        assert!(metadata_object.remove("token_endpoint").is_some());
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        assert!(metadata_object.remove("response_types_supported").is_some());
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        assert!(metadata_object.remove("grant_types_supported").is_some());
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        assert!(metadata_object.remove("revocation_endpoint").is_some());
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object;
        assert!(metadata_object.remove("code_challenge_methods_supported").is_some());
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();
    }

    #[test]
    fn metadata_missing_required_values() {
        let original_metadata_object = authorization_server_metadata_object();

        let mut metadata_object = original_metadata_object.clone();
        get_mut_array(&mut metadata_object, "response_types_supported").clear();
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        get_mut_array(&mut metadata_object, "response_modes_supported").clear();
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        get_mut_array(&mut metadata_object, "response_modes_supported").remove(0);
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        get_mut_array(&mut metadata_object, "response_modes_supported").remove(1);
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        get_mut_array(&mut metadata_object, "grant_types_supported").clear();
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        get_mut_array(&mut metadata_object, "grant_types_supported").remove(0);
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object.clone();
        get_mut_array(&mut metadata_object, "grant_types_supported").remove(1);
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();

        let mut metadata_object = original_metadata_object;
        get_mut_array(&mut metadata_object, "code_challenge_methods_supported").clear();
        from_json_value::<AuthorizationServerMetadata>(metadata_object.into()).unwrap_err();
    }
}
