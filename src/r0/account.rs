//! Endpoints for account registration and management.

pub mod change_password;
pub mod deactivate;
pub mod register;
pub mod request_password_change_token;
pub mod request_register_token;
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
