//! [POST /_matrix/client/r0/login](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-login)

use ruma_api::ruma_api;
use ruma_common::thirdparty::Medium;
use ruma_identifiers::{DeviceId, DeviceIdBox, ServerNameBox, UserId};
use ruma_serde::Outgoing;
use serde::{Deserialize, Serialize};

ruma_api! {
    metadata: {
        description: "Login to the homeserver.",
        method: POST,
        name: "login",
        path: "/_matrix/client/r0/login",
        rate_limited: true,
        authentication: None,
    }

    request: {
        /// Identification information for the user.
        #[serde(flatten)]
        pub user: UserInfo<'a>,

        /// The authentication mechanism.
        #[serde(flatten)]
        pub login_info: LoginInfo<'a>,

        /// ID of the client device
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<&'a DeviceId>,

        /// A display name to assign to the newly-created device. Ignored if device_id corresponds
        /// to a known device.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial_device_display_name: Option<&'a str>,
    }

    response: {
        /// The fully-qualified Matrix ID that has been registered.
        pub user_id: UserId,

        /// An access token for the account.
        pub access_token: String,

        /// The hostname of the homeserver on which the account has been registered.
        ///
        /// Deprecated: Clients should instead use the `user_id.server_name()`
        /// method if they require it.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub home_server: Option<ServerNameBox>,

        /// ID of the logged-in device.
        ///
        /// Will be the same as the corresponding parameter in the request, if one was
        /// specified.
        pub device_id: DeviceIdBox,

        /// Client configuration provided by the server.
        ///
        /// If present, clients SHOULD use the provided object to reconfigure themselves.
        pub well_known: Option<DiscoveryInfo>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user and login info.
    pub fn new(user: UserInfo<'a>, login_info: LoginInfo<'a>) -> Self {
        Self { user, login_info, device_id: None, initial_device_display_name: None }
    }
}

impl Response {
    /// Creates a new `Response` with the given user ID, access token and device ID.
    pub fn new(user_id: UserId, access_token: String, device_id: DeviceIdBox) -> Self {
        Self { user_id, access_token, home_server: None, device_id, well_known: None }
    }
}

/// Identification information for the user.
#[derive(Clone, Debug, PartialEq, Eq, Outgoing, Serialize)]
#[serde(from = "user_serde::IncomingUserInfo", into = "user_serde::UserInfo")]
pub enum UserInfo<'a> {
    /// Either a fully qualified Matrix user ID, or just the localpart (as part of the 'identifier'
    /// field).
    MatrixId(&'a str),

    /// Third party identifier (as part of the 'identifier' field).
    ThirdPartyId {
        /// Third party identifier for the user.
        address: &'a str,

        /// The medium of the identifier.
        medium: Medium,
    },

    /// Same as third-party identification with medium == msisdn, but with a non-canonicalised
    /// phone number.
    PhoneNumber {
        /// The country that the phone number is from.
        country: &'a str,

        /// The phone number.
        phone: &'a str,
    },
}

/// The authentication mechanism.
#[derive(Clone, Debug, PartialEq, Eq, Outgoing, Serialize)]
#[serde(tag = "type")]
pub enum LoginInfo<'a> {
    /// A password is supplied to authenticate.
    #[serde(rename = "m.login.password")]
    Password {
        /// The password.
        password: &'a str,
    },

    /// Token-based login.
    #[serde(rename = "m.login.token")]
    Token {
        /// The token.
        token: &'a str,
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

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use ruma_api::OutgoingRequest;
    use serde_json::{from_value as from_json_value, json, Value as JsonValue};

    use super::{IncomingLoginInfo, IncomingUserInfo, LoginInfo, Medium, Request, UserInfo};

    #[test]
    fn deserialize_login_type() {
        assert_matches!(
            from_json_value(json!({
                "type": "m.login.password",
                "password": "ilovebananas"
            }))
            .unwrap(),
            IncomingLoginInfo::Password { password }
            if password == "ilovebananas"
        );

        assert_matches!(
            from_json_value(json!({
                "type": "m.login.token",
                "token": "1234567890abcdef"
            }))
            .unwrap(),
            IncomingLoginInfo::Token { token }
            if token == "1234567890abcdef"
        );
    }

    #[test]
    fn deserialize_user() {
        assert_matches!(
            from_json_value(json!({
                "identifier": {
                    "type": "m.id.user",
                    "user": "cheeky_monkey"
                }
            }))
            .unwrap(),
            IncomingUserInfo::MatrixId(id)
            if id == "cheeky_monkey"
        );
    }

    #[test]
    fn serialize_login_request_body() {
        let req: http::Request<Vec<u8>> = Request {
            user: UserInfo::ThirdPartyId { address: "hello@example.com", medium: Medium::Email },
            login_info: LoginInfo::Token { token: "0xdeadbeef" },
            device_id: None,
            initial_device_display_name: Some("test"),
        }
        .try_into_http_request("https://homeserver.tld", None)
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
                "type": "m.login.token",
                "token": "0xdeadbeef",
                "initial_device_display_name": "test",
            })
        )
    }
}
