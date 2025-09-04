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
        api::{request, response, Metadata},
        metadata,
        serde::{OrdAsRefStr, PartialEqAsRefStr, PartialOrdAsRefStr, Raw, StringEnum},
    };
    use serde::Serialize;
    use url::Url;

    use crate::PrivOwnedStr;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc2965/auth_metadata",
            1.15 => "/_matrix/client/v1/auth_metadata",
        }
    };

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
        #[cfg(feature = "unstable-msc4191")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub account_management_uri: Option<Url>,

        /// List of actions that the account management URL supports ([MSC4191]).
        ///
        /// [MSC4191]: https://github.com/matrix-org/matrix-spec-proposals/pull/4191
        #[cfg(feature = "unstable-msc4191")]
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
                #[cfg(feature = "unstable-msc4191")]
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
    }

    /// The method to use at the authorization endpoint.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum, PartialEqAsRefStr, Eq, PartialOrdAsRefStr, OrdAsRefStr)]
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
    #[derive(Clone, StringEnum, PartialEqAsRefStr, Eq, PartialOrdAsRefStr, OrdAsRefStr)]
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
    #[derive(Clone, StringEnum, PartialEqAsRefStr, Eq, PartialOrdAsRefStr, OrdAsRefStr)]
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
    #[derive(Clone, StringEnum, PartialEqAsRefStr, Eq, PartialOrdAsRefStr, OrdAsRefStr)]
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
    /// The values are specified in [MSC 4191].
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    ///
    /// [MSC 4191]: https://github.com/matrix-org/matrix-spec-proposals/pull/4191
    #[cfg(feature = "unstable-msc4191")]
    #[derive(Clone, StringEnum, PartialEqAsRefStr, Eq, PartialOrdAsRefStr, OrdAsRefStr)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum AccountManagementAction {
        /// The user wishes to view their profile (name, avatar, contact details).
        ///
        /// [RFC 7636]: https://datatracker.ietf.org/doc/html/rfc7636
        #[ruma_enum(rename = "org.matrix.profile")]
        Profile,

        /// The user wishes to view a list of their sessions.
        #[ruma_enum(rename = "org.matrix.sessions_list")]
        SessionsList,

        /// The user wishes to view the details of a specific session.
        #[ruma_enum(rename = "org.matrix.session_view")]
        SessionView,

        /// The user wishes to end/logout a specific session.
        #[ruma_enum(rename = "org.matrix.session_end")]
        SessionEnd,

        /// The user wishes to deactivate their account.
        #[ruma_enum(rename = "org.matrix.account_deactivate")]
        AccountDeactivate,

        /// The user wishes to reset their cross-signing keys.
        #[ruma_enum(rename = "org.matrix.cross_signing_reset")]
        CrossSigningReset,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
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
    #[derive(Clone, StringEnum, PartialEqAsRefStr, Eq, PartialOrdAsRefStr, OrdAsRefStr)]
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
    use serde_json::{from_value as from_json_value, json, Value as JsonValue};
    use url::Url;

    use super::v1::AuthorizationServerMetadata;

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
                "org.matrix.sessions_list",
                "org.matrix.session_view",
                "org.matrix.session_end",
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

        #[cfg(feature = "unstable-msc4191")]
        {
            let mut metadata = original_metadata.clone();
            metadata.account_management_uri =
                Some(Url::parse("http://server.local/account").unwrap());
            metadata.validate_urls().unwrap_err();
            metadata.insecure_validate_urls().unwrap();
        }

        #[cfg(feature = "unstable-msc4108")]
        {
            let mut metadata = original_metadata.clone();
            metadata.device_authorization_endpoint =
                Some(Url::parse("http://server.local/device").unwrap());
            metadata.validate_urls().unwrap_err();
            metadata.insecure_validate_urls().unwrap();
        }
    }
}
