//! `POST /_matrix/client/*/login`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3login

    use ruma_common::{
        api::ruma_api,
        serde::{Incoming, JsonObject},
        DeviceId, OwnedDeviceId, OwnedServerName, OwnedUserId,
    };
    use serde::{
        de::{self, DeserializeOwned},
        Deserialize, Deserializer, Serialize,
    };
    use serde_json::Value as JsonValue;

    use crate::uiaa::{IncomingUserIdentifier, UserIdentifier};

    ruma_api! {
        metadata: {
            description: "Login to the homeserver.",
            method: POST,
            name: "login",
            r0_path: "/_matrix/client/r0/login",
            stable_path: "/_matrix/client/v3/login",
            rate_limited: true,
            authentication: None,
            added: 1.0,
        }

        request: {
            /// The authentication mechanism.
            #[serde(flatten)]
            pub login_info: LoginInfo<'a>,

            /// ID of the client device
            #[serde(skip_serializing_if = "Option::is_none")]
            pub device_id: Option<&'a DeviceId>,

            /// A display name to assign to the newly-created device.
            ///
            /// Ignored if `device_id` corresponds to a known device.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub initial_device_display_name: Option<&'a str>,
        }

        response: {
            /// The fully-qualified Matrix ID that has been registered.
            pub user_id: OwnedUserId,

            /// An access token for the account.
            pub access_token: String,

            /// The hostname of the homeserver on which the account has been registered.
            ///
            /// Deprecated: Clients should instead use the `user_id.server_name()`
            /// method if they require it.
            #[serde(skip_serializing_if = "Option::is_none")]
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
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given login info.
        pub fn new(login_info: LoginInfo<'a>) -> Self {
            Self { login_info, device_id: None, initial_device_display_name: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given user ID, access token and device ID.
        pub fn new(user_id: OwnedUserId, access_token: String, device_id: OwnedDeviceId) -> Self {
            Self { user_id, access_token, home_server: None, device_id, well_known: None }
        }
    }

    /// The authentication mechanism.
    ///
    /// To construct the custom `LoginInfo` variant you first have to construct
    /// [`IncomingLoginInfo::new`] and then call [`IncomingLoginInfo::to_outgoing`] on it.
    #[derive(Clone, Debug, Incoming, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[incoming_derive(!Deserialize)]
    #[serde(untagged)]
    pub enum LoginInfo<'a> {
        /// An identifier and password are supplied to authenticate.
        Password(Password<'a>),

        /// Token-based login.
        Token(Token<'a>),

        /// Application Service-specific login.
        ApplicationService(ApplicationService<'a>),

        #[doc(hidden)]
        _Custom(CustomLoginInfo<'a>),
    }

    impl IncomingLoginInfo {
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
                _ => Self::_Custom(IncomingCustomLoginInfo {
                    login_type: login_type.into(),
                    extra: data,
                }),
            })
        }

        /// Convert `IncomingLoginInfo` to `LoginInfo`.
        pub fn to_outgoing(&self) -> LoginInfo<'_> {
            match self {
                Self::Password(a) => LoginInfo::Password(a.to_outgoing()),
                Self::Token(a) => LoginInfo::Token(a.to_outgoing()),
                Self::ApplicationService(a) => LoginInfo::ApplicationService(a.to_outgoing()),
                Self::_Custom(a) => LoginInfo::_Custom(CustomLoginInfo {
                    login_type: &a.login_type,
                    extra: &a.extra,
                }),
            }
        }
    }

    impl<'de> Deserialize<'de> for IncomingLoginInfo {
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
                _ => from_json_value(json).map(Self::_Custom),
            }
        }
    }

    /// An identifier and password to supply as authentication.
    #[derive(Clone, Debug, Incoming, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[serde(tag = "type", rename = "m.login.password")]
    pub struct Password<'a> {
        /// Identification information for the user.
        pub identifier: UserIdentifier<'a>,

        /// The password.
        pub password: &'a str,
    }

    impl<'a> Password<'a> {
        /// Creates a new `Password` with the given identifier and password.
        pub fn new(identifier: UserIdentifier<'a>, password: &'a str) -> Self {
            Self { identifier, password }
        }
    }

    impl IncomingPassword {
        /// Convert `IncomingPassword` to `Password`.
        fn to_outgoing(&self) -> Password<'_> {
            Password { identifier: self.identifier.to_outgoing(), password: &self.password }
        }
    }

    /// A token to supply as authentication.
    #[derive(Clone, Debug, Incoming, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[serde(tag = "type", rename = "m.login.token")]
    pub struct Token<'a> {
        /// The token.
        pub token: &'a str,
    }

    impl<'a> Token<'a> {
        /// Creates a new `Token` with the given token.
        pub fn new(token: &'a str) -> Self {
            Self { token }
        }
    }

    impl IncomingToken {
        /// Convert `IncomingToken` to `Token`.
        fn to_outgoing(&self) -> Token<'_> {
            Token { token: &self.token }
        }
    }

    /// An identifier to supply for Application Service authentication.
    #[derive(Clone, Debug, Incoming, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[serde(tag = "type", rename = "m.login.application_service")]
    pub struct ApplicationService<'a> {
        /// Identification information for the user.
        pub identifier: UserIdentifier<'a>,
    }

    impl<'a> ApplicationService<'a> {
        /// Creates a new `ApplicationService` with the given identifier.
        pub fn new(identifier: UserIdentifier<'a>) -> Self {
            Self { identifier }
        }
    }

    impl IncomingApplicationService {
        /// Convert `IncomingApplicationService` to `ApplicationService`.
        fn to_outgoing(&self) -> ApplicationService<'_> {
            ApplicationService { identifier: self.identifier.to_outgoing() }
        }
    }

    #[doc(hidden)]
    #[derive(Clone, Debug, Serialize)]
    #[non_exhaustive]
    pub struct CustomLoginInfo<'a> {
        #[serde(rename = "type")]
        login_type: &'a str,
        #[serde(flatten)]
        extra: &'a JsonObject,
    }

    #[doc(hidden)]
    #[derive(Clone, Debug, Deserialize)]
    #[non_exhaustive]
    pub struct IncomingCustomLoginInfo {
        #[serde(rename = "type")]
        login_type: String,
        #[serde(flatten)]
        extra: JsonObject,
    }

    /// Client configuration provided by the server.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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
        use assert_matches::assert_matches;
        use serde_json::{from_value as from_json_value, json};

        use super::{IncomingLoginInfo, IncomingPassword, IncomingToken};
        use crate::uiaa::IncomingUserIdentifier;

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
                IncomingLoginInfo::Password(IncomingPassword { identifier: IncomingUserIdentifier::UserIdOrLocalpart(user), password })
                if user == "cheeky_monkey" && password == "ilovebananas"
            );

            assert_matches!(
                from_json_value(json!({
                    "type": "m.login.token",
                    "token": "1234567890abcdef"
                }))
                .unwrap(),
                IncomingLoginInfo::Token(IncomingToken { token })
                if token == "1234567890abcdef"
            );
        }

        #[test]
        #[cfg(feature = "client")]
        fn serialize_login_request_body() {
            use ruma_common::{
                api::{MatrixVersion, OutgoingRequest, SendAccessToken},
                thirdparty::Medium,
            };
            use serde_json::Value as JsonValue;

            use super::{LoginInfo, Password, Request, Token};
            use crate::uiaa::UserIdentifier;

            let req: http::Request<Vec<u8>> = Request {
                login_info: LoginInfo::Token(Token { token: "0xdeadbeef" }),
                device_id: None,
                initial_device_display_name: Some("test"),
            }
            .try_into_http_request(
                "https://homeserver.tld",
                SendAccessToken::None,
                &[MatrixVersion::V1_1],
            )
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
                login_info: LoginInfo::Password(Password {
                    identifier: UserIdentifier::ThirdPartyId {
                        address: "hello@example.com",
                        medium: Medium::Email,
                    },
                    password: "deadbeef",
                }),
                device_id: None,
                initial_device_display_name: Some("test"),
            }
            .try_into_http_request(
                "https://homeserver.tld",
                SendAccessToken::None,
                &[MatrixVersion::V1_1],
            )
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
