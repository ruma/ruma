//! `POST /_matrix/client/*/register`

use serde::{Deserialize, Serialize};

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3register

    use ruma_common::{api::ruma_api, DeviceId, OwnedDeviceId, OwnedUserId};

    use super::{LoginType, RegistrationKind};
    use crate::uiaa::{AuthData, IncomingAuthData, UiaaResponse};

    ruma_api! {
        metadata: {
            description: "Register an account on this homeserver.",
            method: POST,
            name: "register",
            r0_path: "/_matrix/client/r0/register",
            stable_path: "/_matrix/client/v3/register",
            rate_limited: true,
            authentication: None,
            added: 1.0,
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
            #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
            pub kind: RegistrationKind,

            /// If `true`, an `access_token` and `device_id` should not be returned
            /// from this call, therefore preventing an automatic login.
            #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
            pub inhibit_login: bool,

            /// Login `type` used by Appservices.
            ///
            /// Appservices can [bypass the registration flows][admin] entirely by providing their
            /// token in the header and setting this login `type` to `m.login.application_service`.
            ///
            /// [admin]: https://spec.matrix.org/v1.2/application-service-api/#server-admin-style-permissions
            #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
            pub login_type: Option<&'a LoginType>,
        }

        response: {
            /// An access token for the account.
            ///
            /// This access token can then be used to authorize other requests.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub access_token: Option<String>,

            /// The fully-qualified Matrix ID that has been registered.
            pub user_id: OwnedUserId,

            /// ID of the registered device.
            ///
            /// Will be the same as the corresponding parameter in the request, if one was specified.
            pub device_id: Option<OwnedDeviceId>,
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
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { access_token: None, user_id, device_id: None }
        }
    }
}

/// The kind of account being registered.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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

/// The login type.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum LoginType {
    /// An appservice-specific login type
    #[serde(rename = "m.login.application_service")]
    ApplicationService,
}
