//! `GET /.well-known/matrix/support` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.18/client-server-api/#getwell-knownmatrixsupport
//!
//! Get server admin contact and support page of a homeserver's domain.

use ruma_common::{
    OwnedUserId,
    api::{auth_scheme::NoAccessToken, request, response},
    metadata,
    serde::StringEnum,
};
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

metadata! {
    method: GET,
    rate_limited: false,
    authentication: NoAccessToken,
    path: "/.well-known/matrix/support",
}

/// Request type for the `discover_support` endpoint.
#[request]
#[derive(Default)]
pub struct Request {}

/// Response type for the `discover_support` endpoint.
#[response]
pub struct Response {
    /// Ways to contact the server administrator.
    ///
    /// At least one of `contacts` or `support_page` is required. If only `contacts` is set, it
    /// must contain at least one item.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contacts: Vec<Contact>,

    /// The URL of a page to give users help specific to the homeserver, like extra
    /// login/registration steps.
    ///
    /// At least one of `contacts` or `support_page` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_page: Option<String>,
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates a new `Response` with the given contacts.
    pub fn with_contacts(contacts: Vec<Contact>) -> Self {
        Self { contacts, support_page: None }
    }

    /// Creates a new `Response` with the given support page.
    pub fn with_support_page(support_page: String) -> Self {
        Self { contacts: Vec::new(), support_page: Some(support_page) }
    }
}

/// A way to contact the server administrator.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct Contact {
    /// An informal description of what the contact methods are used for.
    pub role: ContactRole,

    /// An email address to reach the administrator.
    ///
    /// At least one of `matrix_id` or `email_address` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_address: Option<String>,

    /// A Matrix User ID representing the administrator.
    ///
    /// It could be an account registered on a different homeserver so the administrator can be
    /// contacted when the homeserver is down.
    ///
    /// At least one of `matrix_id` or `email_address` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matrix_id: Option<OwnedUserId>,

    /// An optional URI leading to a PGP key that may be used to encrypt messages sent to the
    /// contact.
    ///
    /// This field uses the unstable prefix defined in [MSC4439].
    ///
    /// [MSC4439]: https://github.com/matrix-org/matrix-spec-proposals/pull/4439
    #[cfg(feature = "unstable-msc4439")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "dev.zirco.msc4439.pgp_key")]
    pub pgp_key: Option<String>,
}

impl Contact {
    /// Creates a new `Contact` with the given role and email address.
    pub fn with_email_address(role: ContactRole, email_address: String) -> Self {
        Self {
            role,
            email_address: Some(email_address),
            matrix_id: None,
            #[cfg(feature = "unstable-msc4439")]
            pgp_key: None,
        }
    }

    /// Creates a new `Contact` with the given role and Matrix User ID.
    pub fn with_matrix_id(role: ContactRole, matrix_id: OwnedUserId) -> Self {
        Self {
            role,
            email_address: None,
            matrix_id: Some(matrix_id),
            #[cfg(feature = "unstable-msc4439")]
            pgp_key: None,
        }
    }
}

/// An informal description of what the contact methods are used for.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all(prefix = "m.role.", rule = "snake_case"))]
#[non_exhaustive]
pub enum ContactRole {
    /// A catch-all role for any queries.
    Admin,

    /// A role intended for sensitive requests.
    Security,

    /// A role for moderation-related queries according to [MSC4121](https://github.com/matrix-org/matrix-spec-proposals/pull/4121).
    ///
    /// The future prefix for this if accepted will be `m.role.moderator`
    #[cfg(feature = "unstable-msc4121")]
    #[ruma_enum(rename = "support.feline.msc4121.role.moderator", alias = "m.role.moderator")]
    Moderator,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
