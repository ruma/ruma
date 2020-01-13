//! Endpoints for account registration and management.

pub mod bind_3pid;
pub mod change_password;
pub mod deactivate;
pub mod delete_3pid;
pub mod get_username_availability;
pub mod register;
pub mod request_3pid_management_token_via_email;
pub mod request_3pid_management_token_via_msisdn;
pub mod request_password_change_token_via_email;
pub mod request_password_change_token_via_msisdn;
pub mod request_registration_token_via_email;
pub mod request_registration_token_via_msisdn;
pub mod unbind_3pid;

pub mod whoami;

use serde::{Deserialize, Serialize};

/// Additional authentication information for the user-interactive authentication API.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthenticationData {
    /// The login type that the client is attempting to complete.
    #[serde(rename = "type")]
    kind: String,
    /// The value of the session key given by the homeserver.
    session: Option<String>,
}

/// Additional authentication information for requestToken endpoints.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentityServerInfo {
    /// The ID server to send the onward request to as a hostname with an
    /// appended colon and port number if the port is not the default.
    pub id_server: String,
    /// Access token previously registered with identity server.
    pub id_access_token: String,
}

/// Possible values for deleting or unbinding 3PIDs
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ThirdPartyIdRemovalStatus {
    NoSupport,
    Success,
}
