//! Endpoints for account registration and management.

pub mod add_3pid;
pub mod bind_3pid;
pub mod change_password;
#[cfg(feature = "unstable-spec")]  // todo: v1.2
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

use ruma_serde::{Outgoing, StringEnum};
use serde::Serialize;

use crate::PrivOwnedStr;

/// Additional authentication information for requestToken endpoints.
#[derive(Clone, Debug, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct IdentityServerInfo<'a> {
    /// The ID server to send the onward request to as a hostname with an
    /// appended colon and port number if the port is not the default.
    pub id_server: &'a str,

    /// Access token previously registered with identity server.
    pub id_access_token: &'a str,
}

impl<'a> IdentityServerInfo<'a> {
    /// Creates a new `IdentityServerInfo` with the given server name and access token.
    pub fn new(id_server: &'a str, id_access_token: &'a str) -> Self {
        Self { id_server, id_access_token }
    }
}

/// Possible values for deleting or unbinding 3PIDs.
///
/// This type can hold an arbitrary string. To check for formats that are not available as a
/// documented variant here, use its string representation, obtained through `.as_str()`.
#[derive(Clone, Debug, StringEnum)]
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

impl ThirdPartyIdRemovalStatus {
    /// Creates a string slice from this `ThirdPartyIdRemovalStatus`.
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
