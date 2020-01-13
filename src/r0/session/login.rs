//! [POST /_matrix/client/r0/login](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-login)

use crate::r0::thirdparty::Medium;
use ruma_api::ruma_api;
use ruma_identifiers::{DeviceId, UserId};
use serde::{Deserialize, Serialize};

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
        /// The user's password.
        pub password: String,
        /// When logging in using a third party identifier, the medium of the identifier.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub medium: Option<Medium>,
        /// The authentication mechanism.
        #[serde(rename = "type")]
        pub login_type: LoginType,
        /// The fully qualified user ID or just local part of the user ID.
        pub user: String,
        /// Third party identifier for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub address: Option<String>,
        /// ID of the client device
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<DeviceId>,
    }

    response {
        /// An access token for the account.
        pub access_token: String,
        /// The hostname of the homeserver on which the account has been registered.
        pub home_server: String,
        /// A refresh token may be exchanged for a new access token using the /tokenrefresh API
        /// endpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub refresh_token: Option<String>,
        /// The fully-qualified Matrix ID that has been registered.
        pub user_id: UserId,
        /// ID of the logged-in device.
        ///
        /// Will be the same as the corresponging parameter in the request, if one was
        /// specified.
        pub device_id: String,
    }
}

/// The authentication mechanism.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum LoginType {
    /// A password is supplied to authenticate.
    #[serde(rename = "m.login.password")]
    Password,
}
