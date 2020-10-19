//! [POST /_matrix/client/r0/register](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-register)

use ruma_api::ruma_api;
use ruma_identifiers::{DeviceId, DeviceIdBox, UserId};
use serde::{Deserialize, Serialize};

use crate::r0::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

ruma_api! {
    metadata: {
        description: "Register an account on this homeserver.",
        method: POST,
        name: "register",
        path: "/_matrix/client/r0/register",
        rate_limited: true,
        authentication: None,
    }

    #[derive(Default)]
    request: {
        /// The desired password for the account.
        ///
        /// May be empty for accounts that should not be able to log in again
        /// with a password, e.g., for guest or application service accounts.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub password: Option<&'a str>,

        /// Localpart of the desired Matrix ID.
        ///
        /// If omitted, the homeserver MUST generate a Matrix ID local part.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub username: Option<&'a str>,

        /// ID of the client device.
        ///
        /// If this does not correspond to a known client device, a new device will be created.
        /// The server will auto-generate a device_id if this is not specified.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<&'a DeviceId>,

        /// A display name to assign to the newly-created device.
        ///
        /// Ignored if `device_id` corresponds to a known device.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial_device_display_name: Option<&'a str>,

        /// Additional authentication information for the user-interactive authentication API.
        ///
        /// Note that this information is not used to define how the registered user should be
        /// authenticated, but is instead used to authenticate the register call itself.
        /// It should be left empty, or omitted, unless an earlier call returned an response
        /// with status code 401.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,

        /// Kind of account to register
        ///
        /// Defaults to `User` if omitted.
        #[ruma_api(query)]
        #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
        pub kind: RegistrationKind,

        /// If `true`, an `access_token` and `device_id` should not be returned
        /// from this call, therefore preventing an automatic login.
        #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
        pub inhibit_login: bool,
    }

    response: {
        /// An access token for the account.
        ///
        /// This access token can then be used to authorize other requests.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub access_token: Option<String>,

        /// The fully-qualified Matrix ID that has been registered.
        pub user_id: UserId,

        /// ID of the registered device.
        ///
        /// Will be the same as the corresponding parameter in the request, if one was specified.
        pub device_id: Option<DeviceIdBox>,
    }

    error: UiaaResponse
}

impl Request<'_> {
    /// Creates a new `Request` with all parameters defaulted.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Response {
    /// Creates a new `Response` with the given user ID.
    pub fn new(user_id: UserId) -> Self {
        Self { access_token: None, user_id, device_id: None }
    }
}

/// The kind of account being registered.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationKind {
    /// A guest account
    ///
    /// These accounts may have limited permissions and may not be supported by all servers.
    Guest,

    /// A regular user account
    User,
}

impl Default for RegistrationKind {
    fn default() -> Self {
        Self::User
    }
}
