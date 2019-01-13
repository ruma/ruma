//! Endpoints for account registration and management.

/// [POST /_matrix/client/r0/register](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-register)
pub mod register {
    use ruma_api_macros::ruma_api;
    use ruma_identifiers::UserId;
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Register an account on this homeserver.",
            method: POST,
            name: "register",
            path: "/_matrix/client/r0/register",
            rate_limited: true,
            requires_authentication: false,
        }

        request {
            /// If true, the server binds the email used for authentication
            /// to the Matrix ID with the ID Server.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub bind_email: Option<bool>,
            /// The desired password for the account.
            ///
            /// Should only be empty for guest accounts.
            // TODO: the spec says nothing about when it is actually required.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub password: Option<String>,
            /// local part of the desired Matrix ID.
            ///
            /// If omitted, the homeserver MUST generate a Matrix ID local part.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub username: Option<String>,
            /// ID of the client device.
            ///
            /// If this does not correspond to a known client device, a new device will be created.
            /// The server will auto-generate a device_id if this is not specified.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub device_id: Option<String>,
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
            pub auth: Option<AuthenticationData>,
            /// Kind of account to register
            ///
            /// Defaults to `User` if ommited.
            #[ruma_api(query)]
            #[serde(skip_serializing_if = "Option::is_none")]
            pub kind: Option<RegistrationKind>,
        }

        response {
            /// An access token for the account.
            ///
            /// This access token can then be used to authorize other requests.
            pub access_token: String,
            /// The hostname of the homeserver on which the account has been registered.
            pub home_server: String,
            /// The fully-qualified Matrix ID that has been registered.
            pub user_id: UserId,
            /// ID of the registered device.
            ///
            /// Will be the same as the corresponding parameter in the request, if one was specified.
            pub device_id: String,
        }
    }

    /// Additional authentication information for the user-interactive authentication API.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct AuthenticationData {
        /// The login type that the client is attempting to complete.
        #[serde(rename = "type")]
        kind: String,
        /// The value of the session key given by the homeserver.
        session: Option<String>,
    }

    /// The kind of account being registered.
    #[derive(Copy, Clone, Debug, Deserialize, Serialize)]
    pub enum RegistrationKind {
        /// A guest account
        ///
        /// These accounts may have limited permissions and may not be supported by all servers.
        #[serde(rename = "guest")]
        Guest,
        /// A regular user account
        #[serde(rename = "user")]
        User,
    }
}

/// [POST /_matrix/client/r0/account/password/email/requestToken](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-password-email-requesttoken)
pub mod request_password_change_token {
    use ruma_api_macros::ruma_api;
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Request that a password change token is sent to the given email address.",
            method: POST,
            name: "request_password_change_token",
            path: "/_matrix/client/r0/account/password/email/requestToken",
            rate_limited: false,
            requires_authentication: false,
        }

        request {
            /// TODO: This parameter is not documented in the spec.
            pub client_secret: String,
            /// TODO: This parameter is not documented in the spec.
            pub email: String,
            /// TODO: This parameter is not documented in the spec.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub id_server: Option<String>,
            /// TODO: This parameter is not documented in the spec.
            pub send_attempt: u64,
        }

        response {}
    }
}

/// [POST /_matrix/client/r0/account/deactivate](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-deactivate)
pub mod deactivate {
    // TODO: missing request parameters

    use ruma_api_macros::ruma_api;

    ruma_api! {
        metadata {
            description: "Deactivate the current user's account.",
            method: POST,
            name: "deactivate",
            path: "/_matrix/client/r0/account/deactivate",
            rate_limited: true,
            requires_authentication: true,
        }

        request {}

        response {}
    }
}

/// [POST /_matrix/client/r0/account/password](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-account-password)
pub mod change_password {
    use ruma_api_macros::ruma_api;
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Change the password of the current user's account.",
            method: POST,
            name: "change_password",
            path: "/_matrix/client/r0/account/password",
            rate_limited: true,
            requires_authentication: true,
        }

        request {
            /// The new password for the account.
            pub new_password: String,
            // TODO: missing `auth` field
        }

        response {}
    }
}

/// [POST /_matrix/client/r0/register/email/requestToken](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-register-email-requesttoken)
pub mod request_register_token {
    use ruma_api_macros::ruma_api;
    use serde_derive::{Deserialize, Serialize};

    ruma_api! {
        metadata {
            description: "Request a register token with a 3rd party email.",
            method: POST,
            name: "request_register_token",
            path: "/_matrix/client/r0/register/email/requestToken",
            rate_limited: true,
            requires_authentication: true,
        }

        request {
            /// Client-generated secret string used to protect this session.
            pub client_secret: String,
            /// The email address.
            pub email: String,
            /// The ID server to send the onward request to as a hostname with an appended colon and port number if the port is not the default.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub id_server: Option<String>,
            /// Used to distinguish protocol level retries from requests to re-send the email.
            pub send_attempt: u64,
        }

        response {}
    }
}
