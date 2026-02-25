//! `GET /_matrix/client/*/auth_metadata`
//!
//! Get the metadata of the authorization server that is trusted by the homeserver.

mod serde;

pub mod v1 {
    //! `v1` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv1auth_metadata

    use std::collections::BTreeSet;

    use ruma_common::{
        DeviceId,
        api::{auth_scheme::NoAuthentication, request, response},
        metadata,
        serde::{Raw, StringEnum},
    };
    use serde::Serialize;
    use url::Url;

    use crate::PrivOwnedStr;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: NoAuthentication,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc2965/auth_metadata",
            1.15 => "/_matrix/client/v1/auth_metadata",
        }
    }

    /// Request type for the `auth_metadata` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `auth_metadata` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The authorization server metadata as defined in [RFC 8414].
        ///
        /// [RFC 8414]: https://datatracker.ietf.org/doc/html/rfc8414
        #[ruma_api(body)]
        pub metadata: Raw<AuthorizationServerMetadata>,
    }

    impl Request {
        /// Creates a new empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given serialized authorization server metadata.
        pub fn new(metadata: Raw<AuthorizationServerMetadata>) -> Self {
            Self { metadata }
        }
    }

    /// Metadata describing the configuration of the authorization server.
    ///
    /// While the metadata properties and their values are declared for OAuth 2.0 in [RFC 8414] and
    /// other RFCs, this type only supports properties and values that are used for Matrix, as
    /// specified in [MSC3861] and its dependencies.
    ///
    /// This type is validated to have at least all the required values during deserialization. The
    /// URLs are not validated during deserialization, to validate them use
    /// [`AuthorizationServerMetadata::validate_urls()`] or
    /// [`AuthorizationServerMetadata::insecure_validate_urls()`].
    ///
    /// This type has no constructor, it should be sent as raw JSON directly.
    ///
    /// [RFC 8414]: https://datatracker.ietf.org/doc/html/rfc8414
    /// [MSC3861]: https://github.com/matrix-org/matrix-spec-proposals/pull/3861
    #[derive(Debug, Clone, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct AuthorizationServerMetadata {
        /// The authorization server's issuer identifier.
        ///
        /// This should be a URL with no query or fragment components.
        pub issuer: Url,

        /// URL of the authorization server's authorization endpoint ([RFC 6749]).
        ///
        /// [RFC 6749]: https://datatracker.ietf.org/doc/html/rfc6749
        pub authorization_endpoint: Url,

        /// URL of the authorization server's token endpoint ([RFC 6749]).
        ///
        /// [RFC 6749]: https://datatracker.ietf.org/doc/html/rfc6749
        pub token_endpoint: Url,

        /// URL of the authorization server's OAuth 2.0 Dynamic Client Registration endpoint
        /// ([RFC 7591]).
        ///
        /// [RFC 7591]: https://datatracker.ietf.org/doc/html/rfc7591
        #[serde(skip_serializing_if = "Option::is_none")]
        pub registration_endpoint: Option<Url>,

        /// List of the OAuth 2.0 `response_type` values that this authorization server supports.
        ///
        /// Those values are the same as those used with the `response_types` parameter defined by
        /// OAuth 2.0 Dynamic Client Registration ([RFC 7591]).
        ///
        /// This field must include [`ResponseType::Code`].
        ///
        /// [RFC 7591]: https://datatracker.ietf.org/doc/html/rfc7591
        pub response_types_supported: BTreeSet<ResponseType>,

        /// List of the OAuth 2.0 `response_mode` values that this authorization server supports.
        ///
        /// Those values are specified in [OAuth 2.0 Multiple Response Type Encoding Practices].
        ///
        /// This field must include [`ResponseMode::Query`] and [`ResponseMode::Fragment`].
        ///
        /// [OAuth 2.0 Multiple Response Type Encoding Practices]: https://openid.net/specs/oauth-v2-multiple-response-types-1_0.html
        pub response_modes_supported: BTreeSet<ResponseMode>,

        /// List of the OAuth 2.0 `grant_type` values that this authorization server supports.
        ///
        /// Those values are the same as those used with the `grant_types` parameter defined by
        /// OAuth 2.0 Dynamic Client Registration ([RFC 7591]).
        ///
        /// This field must include [`GrantType::AuthorizationCode`] and
        /// [`GrantType::RefreshToken`].
        ///
        /// [RFC 7591]: https://datatracker.ietf.org/doc/html/rfc7591
        pub grant_types_supported: BTreeSet<GrantType>,

        /// URL of the authorization server's OAuth 2.0 revocation endpoint ([RFC 7009]).
        ///
        /// [RFC 7009]: https://datatracker.ietf.org/doc/html/rfc7009
        pub revocation_endpoint: Url,

        /// List of Proof Key for Code Exchange (PKCE) code challenge methods supported by this
        /// authorization server ([RFC 7636]).
        ///
        /// This field must include [`CodeChallengeMethod::S256`].
        ///
        /// [RFC 7636]: https://datatracker.ietf.org/doc/html/rfc7636
        pub code_challenge_methods_supported: BTreeSet<CodeChallengeMethod>,

        /// URL where the user is able to access the account management capabilities of the
        /// authorization server ([MSC4191]).
        ///
        /// [MSC4191]: https://github.com/matrix-org/matrix-spec-proposals/pull/4191
        #[serde(skip_serializing_if = "Option::is_none")]
        pub account_management_uri: Option<Url>,

        /// List of actions that the account management URL supports ([MSC4191]).
        ///
        /// [MSC4191]: https://github.com/matrix-org/matrix-spec-proposals/pull/4191
        #[serde(skip_serializing_if = "BTreeSet::is_empty")]
        pub account_management_actions_supported: BTreeSet<AccountManagementAction>,

        /// URL of the authorization server's device authorization endpoint ([RFC 8628]).
        ///
        /// [RFC 8628]: https://datatracker.ietf.org/doc/html/rfc8628
        #[cfg(feature = "unstable-msc4108")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_authorization_endpoint: Option<Url>,

        /// The [`Prompt`] values supported by the authorization server ([Initiating User
        /// Registration via OpenID Connect 1.0]).
        ///
        /// [Initiating User Registration via OpenID Connect 1.0]: https://openid.net/specs/openid-connect-prompt-create-1_0.html
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub prompt_values_supported: Vec<Prompt>,
    }

    impl AuthorizationServerMetadata {
        /// Strict validation of the URLs in this `AuthorizationServerMetadata`.
        ///
        /// This checks that:
        ///
        /// * The `issuer` is a valid URL using an `https` scheme and without a query or fragment.
        ///
        /// * All the URLs use an `https` scheme.
        pub fn validate_urls(&self) -> Result<(), AuthorizationServerMetadataUrlError> {
            self.validate_urls_inner(false)
        }

        /// Weak validation the URLs `AuthorizationServerMetadata` are all absolute URLs.
        ///
        /// This only checks that the `issuer` is a valid URL without a query or fragment.
        ///
        /// In production, you should prefer [`AuthorizationServerMetadata`] that also check if the
        /// URLs use an `https` scheme. This method is meant for development purposes, when
        /// interacting with a local authorization server.
        pub fn insecure_validate_urls(&self) -> Result<(), AuthorizationServerMetadataUrlError> {
            self.validate_urls_inner(true)
        }

        /// Get an iterator over the URLs of this `AuthorizationServerMetadata`, except the
        /// `issuer`.
        fn validate_urls_inner(
            &self,
            insecure: bool,
        ) -> Result<(), AuthorizationServerMetadataUrlError> {
            if self.issuer.query().is_some() || self.issuer.fragment().is_some() {
                return Err(AuthorizationServerMetadataUrlError::IssuerHasQueryOrFragment);
            }

            if insecure {
                // No more checks.
                return Ok(());
            }

            let required_urls = &[
                ("issuer", &self.issuer),
                ("authorization_endpoint", &self.authorization_endpoint),
                ("token_endpoint", &self.token_endpoint),
                ("revocation_endpoint", &self.revocation_endpoint),
            ];
            let optional_urls = &[
                self.registration_endpoint.as_ref().map(|string| ("registration_endpoint", string)),
                self.account_management_uri
                    .as_ref()
                    .map(|string| ("account_management_uri", string)),
                #[cfg(feature = "unstable-msc4108")]
                self.device_authorization_endpoint
                    .as_ref()
                    .map(|string| ("device_authorization_endpoint", string)),
            ];

            for (field, url) in required_urls.iter().chain(optional_urls.iter().flatten()) {
                if url.scheme() != "https" {
                    return Err(AuthorizationServerMetadataUrlError::NotHttpsScheme(field));
                }
            }

            Ok(())
        }

        /// Whether the given account management action is advertised as supported by the server.
        ///
        /// This function tries to be backwards compatible with unstable implementations by checking
        /// both the stable and unstable values of the given action, if they differ.
        pub fn is_account_management_action_supported(
            &self,
            action: &AccountManagementAction,
        ) -> bool {
            match action {
                AccountManagementAction::UnstableSessionsList
                | AccountManagementAction::DevicesList => {
                    self.account_management_actions_supported
                        .contains(&AccountManagementAction::DevicesList)
                        || self
                            .account_management_actions_supported
                            .contains(&AccountManagementAction::UnstableSessionsList)
                }
                AccountManagementAction::UnstableSessionView
                | AccountManagementAction::DeviceView => {
                    self.account_management_actions_supported
                        .contains(&AccountManagementAction::DeviceView)
                        || self
                            .account_management_actions_supported
                            .contains(&AccountManagementAction::UnstableSessionView)
                }
                AccountManagementAction::UnstableSessionEnd
                | AccountManagementAction::DeviceDelete => {
                    self.account_management_actions_supported
                        .contains(&AccountManagementAction::DeviceDelete)
                        || self
                            .account_management_actions_supported
                            .contains(&AccountManagementAction::UnstableSessionEnd)
                }
                action => self.account_management_actions_supported.contains(action),
            }
        }

        /// Build the account management URL with the given action.
        ///
        /// This function tries to be backwards compatible with unstable implementations by
        /// selecting the proper action value to add to the URL (stable or unstable) given
        /// the supported actions advertised in this metadata. If the action is not present
        /// in the metadata, the stable value is used.
        ///
        /// Returns `None` if the `account_management_uri` is `None`.
        pub fn account_management_url_with_action(
            &self,
            action: AccountManagementActionData<'_>,
        ) -> Option<Url> {
            const QUERY_NAME_ACTION: &str = "action";
            const QUERY_NAME_DEVICE_ID: &str = "device_id";

            let mut url = self.account_management_uri.clone()?;

            match action {
                AccountManagementActionData::Profile => {
                    url.query_pairs_mut()
                        .append_pair(QUERY_NAME_ACTION, AccountManagementAction::Profile.as_str());
                }
                AccountManagementActionData::DevicesList => {
                    // Prefer the stable action if it is supported, and default to the stable action
                    // if no actions are advertised in the metadata.
                    let action = if self
                        .account_management_actions_supported
                        .contains(&AccountManagementAction::DevicesList)
                    {
                        AccountManagementAction::DevicesList
                    } else if self
                        .account_management_actions_supported
                        .contains(&AccountManagementAction::UnstableSessionsList)
                    {
                        AccountManagementAction::UnstableSessionsList
                    } else {
                        AccountManagementAction::DevicesList
                    };

                    url.query_pairs_mut().append_pair(QUERY_NAME_ACTION, action.as_str());
                }
                AccountManagementActionData::DeviceView(DeviceViewData { device_id }) => {
                    // Prefer the stable action if it is supported, and default to the stable action
                    // if no actions are advertised in the metadata.
                    let action = if self
                        .account_management_actions_supported
                        .contains(&AccountManagementAction::DeviceView)
                    {
                        AccountManagementAction::DeviceView
                    } else if self
                        .account_management_actions_supported
                        .contains(&AccountManagementAction::UnstableSessionView)
                    {
                        AccountManagementAction::UnstableSessionView
                    } else {
                        AccountManagementAction::DeviceView
                    };

                    url.query_pairs_mut()
                        .append_pair(QUERY_NAME_ACTION, action.as_str())
                        .append_pair(QUERY_NAME_DEVICE_ID, device_id.as_str());
                }
                AccountManagementActionData::DeviceDelete(DeviceDeleteData { device_id }) => {
                    // Prefer the stable action if it is supported, and default to the stable action
                    // if no actions are advertised in the metadata.
                    let action = if self
                        .account_management_actions_supported
                        .contains(&AccountManagementAction::DeviceDelete)
                    {
                        AccountManagementAction::DeviceDelete
                    } else if self
                        .account_management_actions_supported
                        .contains(&AccountManagementAction::UnstableSessionEnd)
                    {
                        AccountManagementAction::UnstableSessionEnd
                    } else {
                        AccountManagementAction::DeviceDelete
                    };

                    url.query_pairs_mut()
                        .append_pair(QUERY_NAME_ACTION, action.as_str())
                        .append_pair(QUERY_NAME_DEVICE_ID, device_id.as_str());
                }
                AccountManagementActionData::AccountDeactivate => {
                    url.query_pairs_mut().append_pair(
                        QUERY_NAME_ACTION,
                        AccountManagementAction::AccountDeactivate.as_str(),
                    );
                }
                AccountManagementActionData::CrossSigningReset => {
                    url.query_pairs_mut().append_pair(
                        QUERY_NAME_ACTION,
                        AccountManagementAction::CrossSigningReset.as_str(),
                    );
                }
            }

            Some(url)
        }
    }

    /// The method to use at the authorization endpoint.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum)]
    #[ruma_enum(rename_all = "lowercase")]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum ResponseType {
        /// Use the authorization code grant flow ([RFC 6749]).
        ///
        /// [RFC 6749]: https://datatracker.ietf.org/doc/html/rfc6749
        Code,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// The mechanism to be used for returning authorization response parameters from the
    /// authorization endpoint.
    ///
    /// The values are specified in [OAuth 2.0 Multiple Response Type Encoding Practices].
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    ///
    /// [OAuth 2.0 Multiple Response Type Encoding Practices]: https://openid.net/specs/oauth-v2-multiple-response-types-1_0.html
    #[derive(Clone, StringEnum)]
    #[ruma_enum(rename_all = "lowercase")]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum ResponseMode {
        /// Authorization Response parameters are encoded in the fragment added to the
        /// `redirect_uri` when redirecting back to the client.
        Query,

        /// Authorization Response parameters are encoded in the query string added to the
        /// `redirect_uri` when redirecting back to the client.
        Fragment,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// The grant type to use at the token endpoint.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum)]
    #[ruma_enum(rename_all = "snake_case")]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum GrantType {
        /// The authorization code grant type ([RFC 6749]).
        ///
        /// [RFC 6749]: https://datatracker.ietf.org/doc/html/rfc6749
        AuthorizationCode,

        /// The refresh token grant type ([RFC 6749]).
        ///
        /// [RFC 6749]: https://datatracker.ietf.org/doc/html/rfc6749
        RefreshToken,

        /// The device code grant type ([RFC 8628]).
        ///
        /// [RFC 8628]: https://datatracker.ietf.org/doc/html/rfc8628
        #[cfg(feature = "unstable-msc4108")]
        #[ruma_enum(rename = "urn:ietf:params:oauth:grant-type:device_code")]
        DeviceCode,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// The code challenge method to use at the authorization endpoint.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum CodeChallengeMethod {
        /// Use a SHA-256, base64url-encoded code challenge ([RFC 7636]).
        ///
        /// [RFC 7636]: https://datatracker.ietf.org/doc/html/rfc7636
        S256,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// The action that the user wishes to do at the account management URL.
    ///
    /// This enum supports both the values that were first specified in [MSC4191] and the values
    /// that replaced them in the Matrix specification, for backwards compatibility with unstable
    /// implementations. The variants that were replaced all use an `Unstable` prefix.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    ///
    /// [MSC4191]: https://github.com/matrix-org/matrix-spec-proposals/pull/4191
    #[derive(Clone, StringEnum)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum AccountManagementAction {
        /// The user wishes to view or edit their profile (name, avatar, contact details).
        #[ruma_enum(rename = "org.matrix.profile")]
        Profile,

        /// The unstable version of [`AccountManagementAction::DevicesList`].
        ///
        /// This uses the `org.matrix.sessions_list` string that was first specified in [MSC4191]
        /// before being replaced by `org.matrix.devices_list`.
        ///
        /// [MSC4191]: https://github.com/matrix-org/matrix-spec-proposals/pull/4191
        #[ruma_enum(rename = "org.matrix.sessions_list")]
        UnstableSessionsList,

        /// The user wishes to view a list of their devices.
        #[ruma_enum(rename = "org.matrix.devices_list")]
        DevicesList,

        /// The unstable version of [`AccountManagementAction::DeviceView`].
        ///
        /// This uses the `org.matrix.session_view` string that was first specified in [MSC4191]
        /// before being replaced by `org.matrix.device_view`.
        ///
        /// [MSC4191]: https://github.com/matrix-org/matrix-spec-proposals/pull/4191
        #[ruma_enum(rename = "org.matrix.session_view")]
        UnstableSessionView,

        /// The user wishes to view the details of a specific device.
        #[ruma_enum(rename = "org.matrix.device_view")]
        DeviceView,

        /// The unstable version of [`AccountManagementAction::DeviceDelete`].
        ///
        /// This uses the `org.matrix.session_end` string that was first specified in [MSC4191]
        /// before being replaced by `org.matrix.device_delete`.
        ///
        /// [MSC4191]: https://github.com/matrix-org/matrix-spec-proposals/pull/4191
        #[ruma_enum(rename = "org.matrix.session_end")]
        UnstableSessionEnd,

        /// The user wishes to delete/log out a specific device.
        #[ruma_enum(rename = "org.matrix.device_delete")]
        DeviceDelete,

        /// The user wishes to deactivate their account.
        #[ruma_enum(rename = "org.matrix.account_deactivate")]
        AccountDeactivate,

        /// The user wishes to reset their cross-signing keys.
        ///
        /// Servers should use this action in the URL of the [`m.oauth`] UIA type.
        ///
        /// [`m.oauth`]: https://spec.matrix.org/latest/client-server-api/#oauth-authentication
        #[ruma_enum(rename = "org.matrix.cross_signing_reset")]
        CrossSigningReset,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// The action that the user wishes to do at the account management URL with its associated
    /// data.
    ///
    /// [MSC 4191]: https://github.com/matrix-org/matrix-spec-proposals/pull/4191
    #[derive(Debug, Clone)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum AccountManagementActionData<'a> {
        /// The user wishes to view or edit their profile (name, avatar, contact details).
        Profile,

        /// The user wishes to view a list of their devices.
        DevicesList,

        /// The user wishes to view the details of a specific device.
        DeviceView(DeviceViewData<'a>),

        /// The user wishes to delete/log out a specific device.
        DeviceDelete(DeviceDeleteData<'a>),

        /// The user wishes to deactivate their account.
        AccountDeactivate,

        /// The user wishes to reset their cross-signing keys.
        CrossSigningReset,
    }

    /// The data associated with [`AccountManagementAction::DeviceView`].
    #[derive(Debug, Clone)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct DeviceViewData<'a> {
        /// The ID of the device to view.
        pub device_id: &'a DeviceId,
    }

    impl<'a> DeviceViewData<'a> {
        /// Construct a new `DeviceViewData` with the given device ID.
        fn new(device_id: &'a DeviceId) -> Self {
            Self { device_id }
        }
    }

    impl<'a> From<&'a DeviceId> for DeviceViewData<'a> {
        fn from(device_id: &'a DeviceId) -> Self {
            Self::new(device_id)
        }
    }

    /// The data associated with [`AccountManagementAction::DeviceDelete`].
    #[derive(Debug, Clone)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct DeviceDeleteData<'a> {
        /// The ID of the device to delete/log out.
        pub device_id: &'a DeviceId,
    }

    impl<'a> DeviceDeleteData<'a> {
        /// Construct a new `DeviceDeleteData` with the given device ID.
        fn new(device_id: &'a DeviceId) -> Self {
            Self { device_id }
        }
    }

    impl<'a> From<&'a DeviceId> for DeviceDeleteData<'a> {
        fn from(device_id: &'a DeviceId) -> Self {
            Self::new(device_id)
        }
    }

    /// The possible errors when validating URLs of [`AuthorizationServerMetadata`].
    #[derive(Debug, Clone, thiserror::Error)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum AuthorizationServerMetadataUrlError {
        /// The URL of the field does not use the `https` scheme.
        #[error("URL in `{0}` must use the `https` scheme")]
        NotHttpsScheme(&'static str),

        /// The `issuer` URL has a query or fragment component.
        #[error("URL in `issuer` cannot have a query or fragment component")]
        IssuerHasQueryOrFragment,
    }

    /// The desired user experience when using the authorization endpoint.
    #[derive(Clone, StringEnum)]
    #[ruma_enum(rename_all = "lowercase")]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum Prompt {
        /// The user wants to create a new account ([Initiating User Registration via OpenID
        /// Connect 1.0]).
        ///
        /// [Initiating User Registration via OpenID Connect 1.0]: https://openid.net/specs/openid-connect-prompt-create-1_0.html
        Create,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::device_id;
    use serde_json::{Value as JsonValue, from_value as from_json_value, json};
    use url::Url;

    use super::v1::{
        AccountManagementAction, AccountManagementActionData, AuthorizationServerMetadata,
    };

    /// A valid `AuthorizationServerMetadata` with all fields and values, as a JSON object.
    pub(super) fn authorization_server_metadata_json() -> JsonValue {
        json!({
            "issuer": "https://server.local/",
            "authorization_endpoint": "https://server.local/authorize",
            "token_endpoint": "https://server.local/token",
            "registration_endpoint": "https://server.local/register",
            "response_types_supported": ["code"],
            "response_modes_supported": ["query", "fragment"],
            "grant_types_supported": ["authorization_code", "refresh_token"],
            "revocation_endpoint": "https://server.local/revoke",
            "code_challenge_methods_supported": ["S256"],
            "account_management_uri": "https://server.local/account",
            "account_management_actions_supported": [
                "org.matrix.profile",
                "org.matrix.devices_list",
                "org.matrix.device_view",
                "org.matrix.device_delete",
                "org.matrix.account_deactivate",
                "org.matrix.cross_signing_reset",
            ],
            "device_authorization_endpoint": "https://server.local/device",
        })
    }

    /// A valid `AuthorizationServerMetadata`, with valid URLs.
    fn authorization_server_metadata() -> AuthorizationServerMetadata {
        from_json_value(authorization_server_metadata_json()).unwrap()
    }

    #[test]
    fn metadata_valid_urls() {
        let metadata = authorization_server_metadata();
        metadata.validate_urls().unwrap();
        metadata.insecure_validate_urls().unwrap();
    }

    #[test]
    fn metadata_invalid_or_insecure_issuer() {
        let original_metadata = authorization_server_metadata();

        // URL with query string.
        let mut metadata = original_metadata.clone();
        metadata.issuer = Url::parse("https://server.local/?session=1er45elp").unwrap();
        metadata.validate_urls().unwrap_err();
        metadata.insecure_validate_urls().unwrap_err();

        // URL with fragment.
        let mut metadata = original_metadata.clone();
        metadata.issuer = Url::parse("https://server.local/#session").unwrap();
        metadata.validate_urls().unwrap_err();
        metadata.insecure_validate_urls().unwrap_err();

        // Insecure URL.
        let mut metadata = original_metadata;
        metadata.issuer = Url::parse("http://server.local/").unwrap();
        metadata.validate_urls().unwrap_err();
        metadata.insecure_validate_urls().unwrap();
    }

    #[test]
    fn metadata_insecure_urls() {
        let original_metadata = authorization_server_metadata();

        let mut metadata = original_metadata.clone();
        metadata.authorization_endpoint = Url::parse("http://server.local/authorize").unwrap();
        metadata.validate_urls().unwrap_err();
        metadata.insecure_validate_urls().unwrap();

        let mut metadata = original_metadata.clone();
        metadata.token_endpoint = Url::parse("http://server.local/token").unwrap();
        metadata.validate_urls().unwrap_err();
        metadata.insecure_validate_urls().unwrap();

        let mut metadata = original_metadata.clone();
        metadata.registration_endpoint = Some(Url::parse("http://server.local/register").unwrap());
        metadata.validate_urls().unwrap_err();
        metadata.insecure_validate_urls().unwrap();

        let mut metadata = original_metadata.clone();
        metadata.revocation_endpoint = Url::parse("http://server.local/revoke").unwrap();
        metadata.validate_urls().unwrap_err();
        metadata.insecure_validate_urls().unwrap();

        let mut metadata = original_metadata.clone();
        metadata.account_management_uri = Some(Url::parse("http://server.local/account").unwrap());
        metadata.validate_urls().unwrap_err();
        metadata.insecure_validate_urls().unwrap();

        #[cfg(feature = "unstable-msc4108")]
        {
            let mut metadata = original_metadata.clone();
            metadata.device_authorization_endpoint =
                Some(Url::parse("http://server.local/device").unwrap());
            metadata.validate_urls().unwrap_err();
            metadata.insecure_validate_urls().unwrap();
        }
    }

    #[test]
    fn metadata_is_account_management_action_supported() {
        let mut original_metadata = authorization_server_metadata();

        // View profile.
        assert!(
            original_metadata
                .is_account_management_action_supported(&AccountManagementAction::Profile)
        );

        // Remove it.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::Profile);
        assert!(
            !original_metadata
                .is_account_management_action_supported(&AccountManagementAction::Profile)
        );

        // View devices list.
        assert!(
            original_metadata
                .is_account_management_action_supported(&AccountManagementAction::DevicesList)
        );
        assert!(original_metadata.is_account_management_action_supported(
            &AccountManagementAction::UnstableSessionsList
        ));

        // Remove it.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::DevicesList);
        assert!(
            !original_metadata
                .is_account_management_action_supported(&AccountManagementAction::DevicesList)
        );
        assert!(!original_metadata.is_account_management_action_supported(
            &AccountManagementAction::UnstableSessionsList
        ));

        // View device.
        assert!(
            original_metadata
                .is_account_management_action_supported(&AccountManagementAction::DeviceView)
        );
        assert!(
            original_metadata.is_account_management_action_supported(
                &AccountManagementAction::UnstableSessionView
            )
        );

        // Remove it.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::DeviceView);
        assert!(
            !original_metadata
                .is_account_management_action_supported(&AccountManagementAction::DeviceView)
        );
        assert!(
            !original_metadata.is_account_management_action_supported(
                &AccountManagementAction::UnstableSessionView
            )
        );

        // Delete device.
        assert!(
            original_metadata
                .is_account_management_action_supported(&AccountManagementAction::DeviceDelete)
        );
        assert!(
            original_metadata.is_account_management_action_supported(
                &AccountManagementAction::UnstableSessionEnd
            )
        );

        // Remove it.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::DeviceDelete);
        assert!(
            !original_metadata
                .is_account_management_action_supported(&AccountManagementAction::DeviceDelete)
        );
        assert!(
            !original_metadata.is_account_management_action_supported(
                &AccountManagementAction::UnstableSessionEnd
            )
        );
    }

    #[test]
    fn metadata_account_management_url_with_action() {
        let mut original_metadata = authorization_server_metadata();
        let device_id = device_id!("DEVICE");

        // View profile.
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::Profile)
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.profile");

        // View devices list, with only the stable action advertised.
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DevicesList)
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.devices_list");

        // View devices list, with both stable and unstable actions advertised.
        original_metadata
            .account_management_actions_supported
            .insert(AccountManagementAction::UnstableSessionsList);
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DevicesList)
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.devices_list");

        // View devices list, with only the unstable action advertised.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::DevicesList);
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DevicesList)
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.sessions_list");

        // View devices list, with no actions advertised.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::UnstableSessionsList);
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DevicesList)
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.devices_list");

        // View device, with only the stable action advertised.
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DeviceView(
                device_id.into(),
            ))
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.device_view&device_id=DEVICE");

        // View device, with both stable and unstable actions advertised.
        original_metadata
            .account_management_actions_supported
            .insert(AccountManagementAction::UnstableSessionView);
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DeviceView(
                device_id.into(),
            ))
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.device_view&device_id=DEVICE");

        // View device, with only the unstable action advertised.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::DeviceView);
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DeviceView(
                device_id.into(),
            ))
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.session_view&device_id=DEVICE");

        // View device, with no actions advertised.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::UnstableSessionView);
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DeviceView(
                device_id.into(),
            ))
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.device_view&device_id=DEVICE");

        // Delete device, with only the stable action advertised.
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DeviceDelete(
                device_id.into(),
            ))
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.device_delete&device_id=DEVICE");

        // Delete device, with both stable and unstable actions advertised.
        original_metadata
            .account_management_actions_supported
            .insert(AccountManagementAction::UnstableSessionEnd);
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DeviceDelete(
                device_id.into(),
            ))
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.device_delete&device_id=DEVICE");

        // Delete device, with only the unstable action advertised.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::DeviceDelete);
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DeviceDelete(
                device_id.into(),
            ))
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.session_end&device_id=DEVICE");

        // Delete device, with no actions advertised.
        original_metadata
            .account_management_actions_supported
            .remove(&AccountManagementAction::UnstableSessionEnd);
        let url = original_metadata
            .account_management_url_with_action(AccountManagementActionData::DeviceDelete(
                device_id.into(),
            ))
            .unwrap();
        assert_eq!(url.query().unwrap(), "action=org.matrix.device_delete&device_id=DEVICE");
    }
}
