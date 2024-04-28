#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! Types and traits for working with the [Matrix](https://matrix.org) protocol.
//!
//! This crate re-exports things from all of the other ruma crates so you don't
//! have to manually keep all the versions in sync.
//!
//! Which crates are re-exported can be configured through cargo features.
//!
//! > ⚠ Some details might be missing because rustdoc has trouble with re-exports so you may need
//! > to refer to other crates' documentations.
//!
//! > 🛈 For internal consistency, Ruma uses American spelling for variable names. Names may differ
//! > in the serialized representation, as the Matrix specification has a mix of British and
//! > American English.
//!
//! # API features
//!
//! Depending on which parts of Matrix are relevant to you, activate the following features:
//!
//! * `appservice-api` -- Application Service API.
//! * `client-api` -- Client-Server API.
//! * `federation-api` -- Server-Server (Federation) API.
//! * `identity-service-api` -- Identity Service API.
//! * `push-gateway-api` -- Push Gateway API.
//!
//! These features have `client`- and `server`-optimized variants that are enabled respectively
//! with the `-c` and `-s` suffixes. For example:
//!   * `client-api-c` -- The Client-Server API optimized for the client side.
//!   * `client-api-s` -- The Client-Server API optimized for the server side.
//!
//! # Compatibility feature
//!
//! * `compat` increases compatibility with other parts of the Matrix ecosystem, at the expense of
//!   deviating from the specification.
//!
//! # Convenience features
//!
//! These features are only useful if you want to use a method that requires it:
//!
//! * `rand` -- Generate random identifiers.
//! * `markdown` -- Parse markdown to construct messages.
//! * `html` -- Parse HTML to sanitize it or navigate its tree.
//!   * `html-matrix` -- Enables the `matrix` feature of `ruma-html` to parse HTML elements data to
//!     typed data as suggested by the Matrix Specification.
//!
//! # Unstable features
//!
//! By using these features, you opt out of all semver guarantees Ruma otherwise provides:
//!
//! * `unstable-exhaustive-types` -- Most types in Ruma are marked as non-exhaustive to avoid
//!   breaking changes when new fields are added in the specification. This feature compiles all
//!   types as exhaustive.
//! * `unstable-mscXXXX`, where `XXXX` is the MSC number -- Upcoming Matrix features that may be
//!   subject to change or removal.
//! * `unstable-unspecified` -- Undocumented Matrix features that may be subject to change or
//!   removal.
//!
//! # Common features
//!
//! These submodules are usually activated by the API features when needed:
//!
//! * `api`
//! * `events`
//! * `signatures`
//!
//! # `ruma-client` features
//!
//! The `client` feature activates [`ruma::client`][client], and `client-ext-client-api` activates
//! `ruma-client`s `client-api` feature. All other `client-*` features activate the same feature
//! without the `client-` prefix on `ruma-client`. See the crate's documentation for the effect of
//! these features.
//!
//! If you are viewing this on `docs.rs`, you can have a look at the feature dependencies by
//! clicking **Feature flags** in the toolbar at the top.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "client")]
#[doc(inline)]
pub use ruma_client as client;
#[cfg(feature = "events")]
#[doc(inline)]
pub use ruma_events as events;
#[cfg(feature = "html")]
#[doc(inline)]
pub use ruma_html as html;
#[cfg(feature = "server-util")]
#[doc(inline)]
pub use ruma_server_util as server_util;
#[cfg(feature = "signatures")]
#[doc(inline)]
pub use ruma_signatures as signatures;
#[cfg(feature = "state-res")]
#[doc(inline)]
pub use ruma_state_res as state_res;

/// (De)serializable types for various [Matrix APIs][apis] requests and responses and abstractions
/// for them.
///
/// [apis]: https://spec.matrix.org/latest/#matrix-apis
#[cfg(feature = "api")]
pub mod api {
    #[cfg(any(feature = "appservice-api-c", feature = "appservice-api-s"))]
    #[doc(inline)]
    pub use ruma_appservice_api as appservice;
    #[cfg(any(feature = "client-api-c", feature = "client-api-s"))]
    #[doc(inline)]
    pub use ruma_client_api as client;
    // The metadata macro is also exported at the crate root because `#[macro_export]` always
    // places things at the crate root of the defining crate and we do a glob re-export of
    // `ruma_common`, but here is the more logical (preferred) location.
    pub use ruma_common::{api::*, metadata};
    #[cfg(any(feature = "federation-api-c", feature = "federation-api-s"))]
    #[doc(inline)]
    pub use ruma_federation_api as federation;
    #[cfg(any(feature = "identity-service-api-c", feature = "identity-service-api-s"))]
    #[doc(inline)]
    pub use ruma_identity_service_api as identity_service;
    #[cfg(any(feature = "push-gateway-api-c", feature = "push-gateway-api-s"))]
    #[doc(inline)]
    pub use ruma_push_gateway_api as push_gateway;
}

#[doc(no_inline)]
pub use assign::assign;
#[doc(no_inline)]
pub use js_int::{int, uint, Int, UInt};
#[doc(no_inline)]
pub use js_option::JsOption;
#[cfg(feature = "client-ext-client-api")]
pub use ruma_client::Client;
pub use ruma_common::*;
pub use web_time as time;
