//! Endpoints for account registration and management.

pub mod add_3pid;
pub mod bind_3pid;
pub mod change_password;
pub mod check_registration_token_validity;
pub mod deactivate;
pub mod delete_3pid;
pub mod get_3pids;
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

use ruma_common::serde::StringEnum;
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// Additional authentication information for requestToken endpoints.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct IdentityServerInfo {
    /// The ID server to send the onward request to as a hostname with an
    /// appended colon and port number if the port is not the default.
    pub id_server: String,

    /// Access token previously registered with identity server.
    pub id_access_token: String,
}

impl IdentityServerInfo {
    /// Creates a new `IdentityServerInfo` with the given server name and access token.
    pub fn new(id_server: String, id_access_token: String) -> Self {
        Self { id_server, id_access_token }
    }
}

/// Possible values for deleting or unbinding 3PIDs.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ThirdPartyIdRemovalStatus {
    /// Either the homeserver couldn't determine the right identity server to contact, or the
    /// identity server refused the operation.
    NoSupport,

    /// Success.
    Success,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
