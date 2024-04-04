#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! (De)serializable types for the [Matrix Application Service API][appservice-api].
//! These types can be shared by application service and server code.
//!
//! [appservice-api]: https://spec.matrix.org/latest/application-service-api/

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

pub mod event;
pub mod ping;
pub mod query;
pub mod thirdparty;

/// A namespace defined by an application service.
///
/// Used for [appservice registration](https://spec.matrix.org/latest/application-service-api/#registration).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Namespace {
    /// Whether this application service has exclusive access to events within this namespace.
    pub exclusive: bool,

    /// A regular expression defining which values this namespace includes.
    pub regex: String,
}

impl Namespace {
    /// Creates a new `Namespace` with the given exclusivity and regex pattern.
    pub fn new(exclusive: bool, regex: String) -> Self {
        Namespace { exclusive, regex }
    }
}

/// Namespaces defined by an application service.
///
/// Used for [appservice registration](https://spec.matrix.org/latest/application-service-api/#registration).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Namespaces {
    /// Events which are sent from certain users.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<Namespace>,

    /// Events which are sent in rooms with certain room aliases.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<Namespace>,

    /// Events which are sent in rooms with certain room IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rooms: Vec<Namespace>,
}

impl Namespaces {
    /// Creates a new `Namespaces` instance with empty namespaces for `users`,  `aliases` and
    /// `rooms` (none of them are explicitly required)
    pub fn new() -> Self {
        Self::default()
    }
}

/// Information required in the registration yaml file that a homeserver needs.
///
/// To create an instance of this type, first create a `RegistrationInit` and convert it via
/// `Registration::from` / `.into()`.
///
/// Used for [appservice registration](https://spec.matrix.org/latest/application-service-api/#registration).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Registration {
    /// A unique, user - defined ID of the application service which will never change.
    pub id: String,

    /// The URL for the application service.
    ///
    /// Optionally set to `null` if no traffic is required.
    pub url: Option<String>,

    /// A unique token for application services to use to authenticate requests to Homeservers.
    pub as_token: String,

    /// A unique token for Homeservers to use to authenticate requests to application services.
    pub hs_token: String,

    /// The localpart of the user associated with the application service.
    pub sender_localpart: String,

    /// A list of users, aliases and rooms namespaces that the application service controls.
    pub namespaces: Namespaces,

    /// Whether requests from masqueraded users are rate-limited.
    ///
    /// The sender is excluded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limited: Option<bool>,

    /// The external protocols which the application service provides (e.g. IRC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<String>>,
}

/// Initial set of fields of `Registration`.
///
/// This struct will not be updated even if additional fields are added to `Registration` in a new
/// (non-breaking) release of the Matrix specification.
///
/// Used for [appservice registration](https://spec.matrix.org/latest/application-service-api/#registration).
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct RegistrationInit {
    /// A unique, user - defined ID of the application service which will never change.
    pub id: String,

    /// The URL for the application service.
    ///
    /// Optionally set to `null` if no traffic is required.
    pub url: Option<String>,

    /// A unique token for application services to use to authenticate requests to Homeservers.
    pub as_token: String,

    /// A unique token for Homeservers to use to authenticate requests to application services.
    pub hs_token: String,

    /// The localpart of the user associated with the application service.
    pub sender_localpart: String,

    /// A list of users, aliases and rooms namespaces that the application service controls.
    pub namespaces: Namespaces,

    /// Whether requests from masqueraded users are rate-limited.
    ///
    /// The sender is excluded.
    pub rate_limited: Option<bool>,

    /// The external protocols which the application service provides (e.g. IRC).
    pub protocols: Option<Vec<String>>,
}

impl From<RegistrationInit> for Registration {
    fn from(init: RegistrationInit) -> Self {
        let RegistrationInit {
            id,
            url,
            as_token,
            hs_token,
            sender_localpart,
            namespaces,
            rate_limited,
            protocols,
        } = init;
        Self { id, url, as_token, hs_token, sender_localpart, namespaces, rate_limited, protocols }
    }
}
