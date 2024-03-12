//! `POST /_matrix/client/*/register`
//!
//! Register an account on this homeserver.

use serde::{Deserialize, Serialize};

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3register

    use std::time::Duration;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedDeviceId, OwnedUserId,
    };

    use super::{LoginType, RegistrationKind};
    use crate::uiaa::{AuthData, UiaaResponse};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AppserviceToken,
        history: {
            1.0 => "/_matrix/client/r0/register",
            1.1 => "/_matrix/client/v3/register",
        }
    };

    /// Request type for the `register` endpoint.
    #[request(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Request {
        /// The desired password for the account.
        ///
        /// May be empty for accounts that should not be able to log in again
        /// with a password, e.g., for guest or application service accounts.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub password: Option<String>,

        /// Localpart of the desired Matrix ID.
        ///
        /// If omitted, the homeserver MUST generate a Matrix ID local part.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub username: Option<String>,

        /// ID of the client device.
        ///
        /// If this does not correspond to a known client device, a new device will be created.
        /// The server will auto-generate a device_id if this is not specified.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<OwnedDeviceId>,

        /// A display name to assign to the newly-created device.
        ///
        /// Ignored if `device_id` corresponds to a known device.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial_device_display_name: Option<String>,

        /// Additional authentication information for the user-interactive authentication API.
        ///
        /// Note that this information is not used to define how the registered user should be
        /// authenticated, but is instead used to authenticate the register call itself.
        /// It should be left empty, or omitted, unless an earlier call returned an response
        /// with status code 401.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData>,

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
        /// [admin]: https://spec.matrix.org/latest/application-service-api/#server-admin-style-permissions
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        pub login_type: Option<LoginType>,

        /// If set to `true`, the client supports [refresh tokens].
        ///
        /// [refresh tokens]: https://spec.matrix.org/latest/client-server-api/#refreshing-access-tokens
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub refresh_token: bool,
    }

    /// Response type for the `register` endpoint.
    #[response(error = UiaaResponse)]
    pub struct Response {
        /// An access token for the account.
        ///
        /// This access token can then be used to authorize other requests.
        ///
        /// Required if the request's `inhibit_login` was set to `false`.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub access_token: Option<String>,

        /// The fully-qualified Matrix ID that has been registered.
        pub user_id: OwnedUserId,

        /// ID of the registered device.
        ///
        /// Will be the same as the corresponding parameter in the request, if one was specified.
        ///
        /// Required if the request's `inhibit_login` was set to `false`.
        pub device_id: Option<OwnedDeviceId>,

        /// A [refresh token] for the account.
        ///
        /// This token can be used to obtain a new access token when it expires by calling the
        /// [`refresh_token`] endpoint.
        ///
        /// Omitted if the request's `inhibit_login` was set to `true`.
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
        ///
        /// Omitted if the request's `inhibit_login` was set to `true`.
        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
            rename = "expires_in_ms"
        )]
        pub expires_in: Option<Duration>,
    }

    impl Request {
        /// Creates a new `Request` with all parameters defaulted.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Response {
        /// Creates a new `Response` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self {
                access_token: None,
                user_id,
                device_id: None,
                refresh_token: None,
                expires_in: None,
            }
        }
    }
}

/// The kind of account being registered.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum RegistrationKind {
    /// A guest account
    ///
    /// These accounts may have limited permissions and may not be supported by all servers.
    Guest,

    /// A regular user account
    #[default]
    User,
}

/// The login type.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum LoginType {
    /// An appservice-specific login type
    #[serde(rename = "m.login.application_service")]
    ApplicationService,
}
