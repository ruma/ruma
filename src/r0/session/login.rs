//! [POST /_matrix/client/r0/login](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-login)

use ruma_api::ruma_api;
use ruma_identifiers::{DeviceId, UserId};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::r0::thirdparty::Medium;

ruma_api! {
    metadata {
        description: "Login to the homeserver.",
        method: POST,
        name: "login",
        path: "/_matrix/client/r0/login",
        rate_limited: true,
        requires_authentication: false,
    }

    request {
        /// Identification information for the user.
        #[serde(flatten)]
        pub user: UserInfo,
        /// The authentication mechanism.
        #[serde(flatten)]
        pub login_info: LoginInfo,
        /// ID of the client device
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<DeviceId>,
        /// A display name to assign to the newly-created device. Ignored if device_id corresponds
        /// to a known device.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial_device_display_name: Option<String>,
    }

    response {
        /// The fully-qualified Matrix ID that has been registered.
        pub user_id: UserId,
        /// An access token for the account.
        pub access_token: String,
        /// The hostname of the homeserver on which the account has been registered.
        ///
        /// Deprecated: Clients should extract the server_name from user_id (by splitting at the
        /// first colon) if they require it.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub home_server: Option<String>,
        /// ID of the logged-in device.
        ///
        /// Will be the same as the corresponging parameter in the request, if one was
        /// specified.
        pub device_id: String,
        /// Client configuration provided by the server.
        ///
        /// If present, clients SHOULD use the provided object to reconfigure themselves.
        pub well_known: Option<DiscoveryInfo>,
    }

    error: crate::Error
}

/// Identification information for the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserInfo {
    /// Either a fully qualified Matrix user ID, or just the localpart (as part of the 'identifier'
    /// field).
    MatrixId(String),
    /// Third party identifier (as part of the 'identifier' field).
    ThirdPartyId {
        /// Third party identifier for the user.
        address: String,
        /// The medium of the identifier.
        medium: Medium,
    },
    /// Same as third-party identification with medium == msisdn, but with a non-canonicalised
    /// phone number.
    PhoneNumber {
        /// The country that the phone number is from.
        country: String,
        /// The phone number.
        phone: String,
    },
}

/// The authentication mechanism.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum LoginInfo {
    /// A password is supplied to authenticate.
    #[serde(rename = "m.login.password")]
    Password {
        /// The password.
        password: String,
    },
    /// Token-based login.
    #[serde(rename = "m.login.token")]
    Token {
        /// The token.
        token: String,
    },
}

/// Client configuration provided by the server.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveryInfo {
    /// Information about the homeserver to connect to.
    #[serde(rename = "m.homeserver")]
    pub homeserver: HomeserverInfo,
    /// Information about the identity server to connect to.
    #[serde(rename = "m.identity_server")]
    pub identity_server: Option<IdentityServerInfo>,
}

/// Information about the homeserver to connect to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HomeserverInfo {
    /// The base URL for the homeserver for client-server connections.
    pub base_url: String,
}

/// Information about the identity server to connect to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentityServerInfo {
    /// The base URL for the identity server for client-server connections.
    pub base_url: String,
}

mod user_serde;

impl Serialize for UserInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        user_serde::UserInfo::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UserInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        user_serde::UserInfo::deserialize(deserializer).map(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use serde_json::json;

    use super::{LoginInfo, Medium, Request, UserInfo};

    #[test]
    fn deserialize_login_type() {
        assert_eq!(
            serde_json::from_str::<LoginInfo>(
                r#"
                {
                    "type": "m.login.password",
                    "password": "ilovebananas"
                }
                "#,
            )
            .unwrap(),
            LoginInfo::Password {
                password: "ilovebananas".into()
            }
        );

        assert_eq!(
            serde_json::from_str::<LoginInfo>(
                r#"
                {
                    "type": "m.login.token",
                    "token": "1234567890abcdef"
                }
                "#,
            )
            .unwrap(),
            LoginInfo::Token {
                token: "1234567890abcdef".into()
            }
        );
    }

    #[test]
    fn deserialize_user() {
        assert_eq!(
            serde_json::from_str::<UserInfo>(
                r#"
                {
                    "identifier": {
                        "type": "m.id.user",
                        "user": "cheeky_monkey"
                    }
                }
                "#,
            )
            .unwrap(),
            UserInfo::MatrixId("cheeky_monkey".into())
        );
    }

    #[test]
    fn serialize_login_request_body() {
        let req: http::Request<Vec<u8>> = Request {
            user: UserInfo::ThirdPartyId {
                address: "hello@example.com".to_owned(),
                medium: Medium::Email,
            },
            login_info: LoginInfo::Token {
                token: "0xdeadbeef".to_owned(),
            },
            device_id: None,
            initial_device_display_name: Some("test".to_string()),
        }
        .try_into()
        .unwrap();

        let req_body_value: serde_json::Value = serde_json::from_slice(req.body()).unwrap();
        assert_eq!(
            req_body_value,
            json!({
                "identifier": {
                    "type": "m.id.thirdparty",
                    "medium": "email",
                    "address": "hello@example.com"
                },
                "type": "m.login.token",
                "token": "0xdeadbeef",
                "initial_device_display_name": "test",
            })
        )
    }
}
