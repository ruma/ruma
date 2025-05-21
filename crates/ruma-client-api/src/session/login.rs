//! `POST /_matrix/client/*/login`
//!
//! Login to the homeserver.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3login

    use std::{fmt, time::Duration};

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::JsonObject,
        OwnedDeviceId, OwnedServerName, OwnedUserId,
    };
    use serde::{
        de::{self, DeserializeOwned},
        Deserialize, Deserializer, Serialize,
    };
    use serde_json::Value as JsonValue;

    use crate::uiaa::UserIdentifier;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AppserviceTokenOptional,
        history: {
            1.0 => "/_matrix/client/r0/login",
            1.1 => "/_matrix/client/v3/login",
        }
    };

    /// Request type for the `login` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The authentication mechanism.
        #[serde(flatten)]
        pub login_info: LoginInfo,

        /// ID of the client device
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<OwnedDeviceId>,

        /// A display name to assign to the newly-created device.
        ///
        /// Ignored if `device_id` corresponds to a known device.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial_device_display_name: Option<String>,

        /// If set to `true`, the client supports [refresh tokens].
        ///
        /// [refresh tokens]: https://spec.matrix.org/latest/client-server-api/#refreshing-access-tokens
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub refresh_token: bool,
    }

    /// Response type for the `login` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The fully-qualified Matrix ID that has been registered.
        pub user_id: OwnedUserId,

        /// An access token for the account.
        pub access_token: String,

        /// The hostname of the homeserver on which the account has been registered.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[deprecated = "\
            Since Matrix Client-Server API r0.4.0. Clients should instead use the \
            `user_id.server_name()` method if they require it.\
        "]
        pub home_server: Option<OwnedServerName>,

        /// ID of the logged-in device.
        ///
        /// Will be the same as the corresponding parameter in the request, if one was
        /// specified.
        pub device_id: OwnedDeviceId,

        /// Client configuration provided by the server.
        ///
        /// If present, clients SHOULD use the provided object to reconfigure themselves.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub well_known: Option<DiscoveryInfo>,

        /// A [refresh token] for the account.
        ///
        /// This token can be used to obtain a new access token when it expires by calling the
        /// [`refresh_token`] endpoint.
        ///
        /// [refresh token]: https://spec.matrix.org/latest/client-server-api/#refreshing-access-tokens
        /// [`refresh_token`]: crate::session::refresh_token
        #[serde(skip_serializing_if = "Option::is_none")]
        pub refresh_token: Option<String>,

        /// The lifetime of the access token, in milliseconds.
        ///
        /// Once the access token has expired, a new access token can be obtained by using the
        /// provided refresh token. If no refresh token is provided, the client will need to
        /// re-login to obtain a new access token.
        ///
        /// If this is `None`, the client can assume that the access token will not expire.
        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
            rename = "expires_in_ms"
        )]
        pub expires_in: Option<Duration>,
    }
    impl Request {
        /// Creates a new `Request` with the given login info.
        pub fn new(login_info: LoginInfo) -> Self {
            Self {
                login_info,
                device_id: None,
                initial_device_display_name: None,
                refresh_token: false,
            }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given user ID, access token and device ID.
        #[allow(deprecated)]
        pub fn new(user_id: OwnedUserId, access_token: String, device_id: OwnedDeviceId) -> Self {
            Self {
                user_id,
                access_token,
                home_server: None,
                device_id,
                well_known: None,
                refresh_token: None,
                expires_in: None,
            }
        }
    }

    /// The authentication mechanism.
    #[derive(Clone, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[serde(untagged)]
    pub enum LoginInfo {
        /// An identifier and password are supplied to authenticate.
        Password(Password),

        /// Token-based login.
        Token(Token),

        /// Application Service-specific login.
        ApplicationService(ApplicationService),

        #[doc(hidden)]
        _Custom(CustomLoginInfo),
    }

    impl LoginInfo {
        /// Creates a new `IncomingLoginInfo` with the given `login_type` string, session and data.
        ///
        /// Prefer to use the public variants of `IncomingLoginInfo` where possible; this
        /// constructor is meant be used for unsupported authentication mechanisms only and
        /// does not allow setting arbitrary data for supported ones.
        ///
        /// # Errors
        ///
        /// Returns an error if the `login_type` is known and serialization of `data` to the
        /// corresponding `IncomingLoginInfo` variant fails.
        pub fn new(login_type: &str, data: JsonObject) -> serde_json::Result<Self> {
            Ok(match login_type {
                "m.login.password" => {
                    Self::Password(serde_json::from_value(JsonValue::Object(data))?)
                }
                "m.login.token" => Self::Token(serde_json::from_value(JsonValue::Object(data))?),
                "m.login.application_service" => {
                    Self::ApplicationService(serde_json::from_value(JsonValue::Object(data))?)
                }
                _ => Self::_Custom(CustomLoginInfo { login_type: login_type.into(), extra: data }),
            })
        }
    }

    impl fmt::Debug for LoginInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // Print `Password { .. }` instead of `Password(Password { .. })`
            match self {
                Self::Password(inner) => inner.fmt(f),
                Self::Token(inner) => inner.fmt(f),
                Self::ApplicationService(inner) => inner.fmt(f),
                Self::_Custom(inner) => inner.fmt(f),
            }
        }
    }

    impl<'de> Deserialize<'de> for LoginInfo {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            fn from_json_value<T: DeserializeOwned, E: de::Error>(val: JsonValue) -> Result<T, E> {
                serde_json::from_value(val).map_err(E::custom)
            }

            // FIXME: Would be better to use serde_json::value::RawValue, but that would require
            // implementing Deserialize manually for Request, bc. `#[serde(flatten)]` breaks things.
            let json = JsonValue::deserialize(deserializer)?;

            let login_type =
                json["type"].as_str().ok_or_else(|| de::Error::missing_field("type"))?;
            match login_type {
                "m.login.password" => from_json_value(json).map(Self::Password),
                "m.login.token" => from_json_value(json).map(Self::Token),
                "m.login.application_service" => {
                    from_json_value(json).map(Self::ApplicationService)
                }
                _ => from_json_value(json).map(Self::_Custom),
            }
        }
    }

    /// An identifier and password to supply as authentication.
    #[derive(Clone, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[serde(tag = "type", rename = "m.login.password")]
    pub struct Password {
        /// Identification information for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub identifier: Option<UserIdentifier>,

        /// The password.
        pub password: String,

        /// Username for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[deprecated = "\
            Since Matrix Client-Server API r0.4.0, clients should use `identifier`\
            instead.\
        "]
        pub user: Option<String>,

        /// 3rd-party identifier address for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[deprecated = "\
            Since Matrix Client-Server API r0.4.0, clients should use `identifier`\
            instead.\
        "]
        pub address: Option<String>,

        /// 3rd-party identifier medium for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[deprecated = "\
            Since Matrix Client-Server API r0.4.0, clients should use `identifier`\
            instead.\
        "]
        pub medium: Option<String>,
    }

    impl Password {
        /// Creates a new `Password` with the given identifier and password.
        #[allow(deprecated)]
        pub fn new(identifier: UserIdentifier, password: String) -> Self {
            Self { identifier: Some(identifier), password, user: None, address: None, medium: None }
        }
    }

    impl fmt::Debug for Password {
        #[allow(deprecated)]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self { identifier, password: _, user, address, medium } = self;
            f.debug_struct("Password")
                .field("identifier", identifier)
                .field("user", user)
                .field("address", address)
                .field("medium", medium)
                .finish_non_exhaustive()
        }
    }

    /// A token to supply as authentication.
    #[derive(Clone, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[serde(tag = "type", rename = "m.login.token")]
    pub struct Token {
        /// The token.
        pub token: String,
    }

    impl Token {
        /// Creates a new `Token` with the given token.
        pub fn new(token: String) -> Self {
            Self { token }
        }
    }

    impl fmt::Debug for Token {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self { token: _ } = self;
            f.debug_struct("Token").finish_non_exhaustive()
        }
    }

    /// An identifier to supply for Application Service authentication.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[serde(tag = "type", rename = "m.login.application_service")]
    pub struct ApplicationService {
        /// Identification information for the user.
        pub identifier: Option<UserIdentifier>,

        /// Username for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[deprecated = "\
            Since Matrix Client-Server API r0.4.0, clients should use `identifier`\
            instead.\
        "]
        pub user: Option<String>,
    }

    impl ApplicationService {
        /// Creates a new `ApplicationService` with the given identifier.
        #[allow(deprecated)]
        pub fn new(identifier: UserIdentifier) -> Self {
            Self { identifier: Some(identifier), user: None }
        }
    }

    #[doc(hidden)]
    #[derive(Clone, Deserialize, Serialize)]
    #[non_exhaustive]
    pub struct CustomLoginInfo {
        #[serde(rename = "type")]
        login_type: String,
        #[serde(flatten)]
        extra: JsonObject,
    }

    impl fmt::Debug for CustomLoginInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let Self { login_type, extra: _ } = self;
            f.debug_struct("CustomLoginInfo")
                .field("login_type", login_type)
                .finish_non_exhaustive()
        }
    }

    /// Client configuration provided by the server.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct DiscoveryInfo {
        /// Information about the homeserver to connect to.
        #[serde(rename = "m.homeserver")]
        pub homeserver: HomeserverInfo,

        /// Information about the identity server to connect to.
        #[serde(rename = "m.identity_server")]
        pub identity_server: Option<IdentityServerInfo>,
    }

    impl DiscoveryInfo {
        /// Create a new `DiscoveryInfo` with the given homeserver.
        pub fn new(homeserver: HomeserverInfo) -> Self {
            Self { homeserver, identity_server: None }
        }
    }

    /// Information about the homeserver to connect to.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct HomeserverInfo {
        /// The base URL for the homeserver for client-server connections.
        pub base_url: String,
    }

    impl HomeserverInfo {
        /// Create a new `HomeserverInfo` with the given base url.
        pub fn new(base_url: String) -> Self {
            Self { base_url }
        }
    }

    /// Information about the identity server to connect to.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct IdentityServerInfo {
        /// The base URL for the identity server for client-server connections.
        pub base_url: String,
    }

    impl IdentityServerInfo {
        /// Create a new `IdentityServerInfo` with the given base url.
        pub fn new(base_url: String) -> Self {
            Self { base_url }
        }
    }

    #[cfg(test)]
    mod tests {
        use assert_matches2::assert_matches;
        use serde_json::{from_value as from_json_value, json};

        use super::{LoginInfo, Token};
        use crate::uiaa::UserIdentifier;

        #[test]
        fn deserialize_login_type() {
            assert_matches!(
                from_json_value(json!({
                    "type": "m.login.password",
                    "identifier": {
                        "type": "m.id.user",
                        "user": "cheeky_monkey"
                    },
                    "password": "ilovebananas"
                }))
                .unwrap(),
                LoginInfo::Password(login)
            );
            assert_matches!(login.identifier, Some(UserIdentifier::UserIdOrLocalpart(user)));
            assert_eq!(user, "cheeky_monkey");
            assert_eq!(login.password, "ilovebananas");

            assert_matches!(
                from_json_value(json!({
                    "type": "m.login.token",
                    "token": "1234567890abcdef"
                }))
                .unwrap(),
                LoginInfo::Token(Token { token })
            );
            assert_eq!(token, "1234567890abcdef");
        }

        #[test]
        #[cfg(feature = "client")]
        fn serialize_login_request_body() {
            use ruma_common::api::{
                MatrixVersion, OutgoingRequest, SendAccessToken, SupportedVersions,
            };
            use serde_json::Value as JsonValue;

            use super::{LoginInfo, Password, Request, Token};
            use crate::uiaa::UserIdentifier;

            let supported =
                SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Vec::new() };

            let req: http::Request<Vec<u8>> = Request {
                login_info: LoginInfo::Token(Token { token: "0xdeadbeef".to_owned() }),
                device_id: None,
                initial_device_display_name: Some("test".to_owned()),
                refresh_token: false,
            }
            .try_into_http_request("https://homeserver.tld", SendAccessToken::None, &supported)
            .unwrap();

            let req_body_value: JsonValue = serde_json::from_slice(req.body()).unwrap();
            assert_eq!(
                req_body_value,
                json!({
                    "type": "m.login.token",
                    "token": "0xdeadbeef",
                    "initial_device_display_name": "test",
                })
            );

            let req: http::Request<Vec<u8>> = Request {
                #[allow(deprecated)]
                login_info: LoginInfo::Password(Password {
                    identifier: Some(UserIdentifier::Email {
                        address: "hello@example.com".to_owned(),
                    }),
                    password: "deadbeef".to_owned(),
                    user: None,
                    address: None,
                    medium: None,
                }),
                device_id: None,
                initial_device_display_name: Some("test".to_owned()),
                refresh_token: false,
            }
            .try_into_http_request("https://homeserver.tld", SendAccessToken::None, &supported)
            .unwrap();

            let req_body_value: JsonValue = serde_json::from_slice(req.body()).unwrap();
            assert_eq!(
                req_body_value,
                json!({
                    "identifier": {
                        "type": "m.id.thirdparty",
                        "medium": "email",
                        "address": "hello@example.com"
                    },
                    "type": "m.login.password",
                    "password": "deadbeef",
                    "initial_device_display_name": "test",
                })
            );
        }
    }
}
