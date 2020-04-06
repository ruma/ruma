//! Endpoints for account registration and management.

pub mod bind_3pid;
pub mod change_password;
pub mod deactivate;
pub mod delete_3pid;
pub mod get_username_availability;
pub mod register;
pub mod request_3pid_management_token_via_email;
pub mod request_3pid_management_token_via_msisdn;
pub mod request_openid_token;
pub mod request_password_change_token_via_email;
pub mod request_password_change_token_via_msisdn;
pub mod request_registration_token_via_email;
pub mod request_registration_token_via_msisdn;
pub mod unbind_3pid;

pub mod whoami;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Additional authentication information for the user-interactive authentication API.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthenticationData {
    /// The login type that the client is attempting to complete.
    #[serde(rename = "type")]
    pub kind: String,
    /// The value of the session key given by the homeserver.
    pub session: Option<String>,
    /// Parameters submitted for a particular authentication stage.
    #[serde(flatten)]
    pub auth_parameters: BTreeMap<String, serde_json::Value>,
}

/// Information about available authentication flows and status for
/// User-Interactive Authenticiation API.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserInteractiveAuthenticationInfo {
    /// List of authentication flows available for this endpoint.
    pub flows: Vec<AuthenticationFlow>,
    /// List of stages in the current flow completed by the client.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed: Vec<String>,
    /// Authentication parameters required for the client to complete authentication.
    pub params: serde_json::Value,
    /// Session key for client to use to complete authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
}

/// Description of steps required to authenticate via the User-Interactive
/// Authentication API.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuthenticationFlow {
    /// Ordered list of stages required to complete authentication.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<String>,
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
pub enum ThirdPartyIdRemovalStatus {
    /// Either the homeserver couldn't determine the right identity server to contact, or the
    /// identity server refused the operation.
    NoSupport,
    /// Success.
    Success,
}
