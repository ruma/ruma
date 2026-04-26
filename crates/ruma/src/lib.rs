#![doc(html_favicon_url = "https://ruma.dev/favicon.ico")]
#![doc(html_logo_url = "https://ruma.dev/images/logo.png")]
//! Types and traits for working with the [Matrix](https://matrix.org) protocol.
//!
//! # Getting started
//!
//! If you want to build a Matrix client or bot, have a look at [matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk#readme).
//! It builds on Ruma and includes handling of state storage, end-to-end encryption and many other
//! useful things.
//!
//! For homeservers, bridges and harder-to-categorize software that works with Matrix, you're at the
//! right place. To get started, add `ruma` to your dependencies:
//!
//! ```toml
//! # crates.io release
//! ruma = { version = "0.15.0", features = ["..."] }
//! # git dependency
//! ruma = { git = "https://github.com/ruma/ruma", branch = "main", features = ["..."] }
//! ```
//!
//! You can find a low level Matrix client in the [ruma-client repository](https://github.com/ruma/ruma-client).
//!
//! You can also find a small number of examples in our dedicated [ruma-examples repository](https://github.com/ruma/ruma-examples).
//!
//! # Status
//!
//! Ruma supports all events and REST endpoints of Matrix 1.18.
//!
//! Only room versions enforcing canonical JSON (introduced with room version 6) are supported. Room
//! versions 1 through 5 are supported on a best effort basis, but a missing feature or an
//! incompatibility with a homeserver implementation are not considered bugs. Clients should be able
//! to work with those room versions, granted parts of the room might break in some unconventional
//! cases, but homeservers based on Ruma **should not** advertise support for them.
//!
//! # Features
//!
//! This crate re-exports things from all of the other ruma crates so you don't have to manually
//! keep all the versions in sync.
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
//! ## API features
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
//! ## Compatibility features
//!
//! By default, the ruma crates are only able to handle strictly spec-compliant data and behaviour.
//! However, due to the fact that Matrix is federated, that it is used by various implementations
//! that might have different bugs, and that much of its data is immutable, they need to be able to
//! interoperate with data that might differ slightly from the specification.
//!
//! This is the role of the `compat-*` cargo features. They allow the crates be more tolerant of
//! external data and incoming requests for known and reasonable deviations from the spec, usually
//! for historical reasons. They however do not permit the ruma crates to generate data that is not
//! spec-compliant.
//!
//! Each cargo feature is documented briefly in the cargo manifest of the crate, and more thoroughly
//! where the feature applies.
//!
//! ## Convenience features
//!
//! These features are only useful if you want to use a method that requires it:
//!
//! * `rand` -- Generate random identifiers.
//! * `markdown` -- Parse markdown to construct messages.
//! * `html` -- Parse HTML to sanitize it or navigate its tree.
//!   * `html-matrix` -- Enables the `matrix` feature of `ruma-html` to parse HTML elements data to
//!     typed data as suggested by the Matrix Specification.
//!
//! ## Unstable features
//!
//! By using these features, you opt out of all semver guarantees Ruma otherwise provides:
//!
//! * `unstable-mscXXXX`, where `XXXX` is the MSC number -- Upcoming Matrix features that may be
//!   subject to change or removal.
//! * `unstable-uniffi` -- Enables UniFFI bindings by adding conditional `uniffi` derives to _some_
//!   types. This feature is currently a work in progress and, thus, unstable.
//!
//! ## Common features
//!
//! These submodules are usually activated by the API features when needed:
//!
//! * `api`
//! * `events`
//! * `signatures`
//!
//! # Compile-time `cfg` settings
//!
//! These settings are accepted at compile time to configure the generated code. They can be set
//! using the `RUSTFLAGS` environment variable like this:
//!
//! ```shell
//! RUSTFLAGS="--cfg {key}=\"{value}\""
//! ```
//!
//! or in `.cargo/config.toml`:
//!
//! ```toml
//! # General setting for all targets, overridden by per-target `rustflags` setting if set.
//! [build]
//! rustflags = ["--cfg", "{key}=\"{value}\""]
//!
//! # Per-target setting.
//! [target.<triple/cfg>]
//! rustflags = ["--cfg", "{key}=\"{value}\""]
//! ```
//!
//! They can also be configured using an environment variable at compile time, which has the benefit
//! of not requiring to re-compile the whole dependency chain when their value is changed, like
//! this:
//!
//! ```shell
//! {UPPERCASE_KEY}="{value}"
//! ```
//!
//! * `ruma_identifiers_storage` -- Choose the inner representation of the identifier types
//!   generated with the `ruma_id` attribute macro. If the setting is not set or has an unknown
//!   value, the owned identifiers use a `Box<str>` internally. The following values are also
//!   supported:
//!
//!   * `Arc` -- Use an `Arc<str>`.
//!
//!   This setting can also be configured by setting the `RUMA_IDENTIFIERS_STORAGE` environment
//!   variable.
//! * `ruma_unstable_exhaustive_types` -- Most types in Ruma are marked as non-exhaustive to avoid
//!   breaking changes when new fields are added in the specification. This setting compiles all
//!   types as exhaustive. By enabling this feature you opt out of all semver guarantees Ruma
//!   otherwise provides. This can also be configured by setting the
//!   `RUMA_UNSTABLE_EXHAUSTIVE_TYPES` environment variable.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "events")]
#[doc(inline)]
pub use ruma_events as events;
#[cfg(feature = "html")]
#[doc(inline)]
pub use ruma_html as html;
#[cfg(feature = "signatures")]
#[doc(inline)]
pub use ruma_signatures as signatures;
#[cfg(feature = "state-res")]
#[doc(inline)]
pub use ruma_state_res as state_res;

/// (De)serializable types for various [Matrix APIs][apis] requests and responses and abstractions
/// for them.
///
/// [apis]: https://spec.matrix.org/v1.18/#matrix-apis
#[cfg(feature = "api")]
pub mod api {
    #[cfg(any(feature = "appservice-api-c", feature = "appservice-api-s"))]
    #[doc(inline)]
    pub use ruma_appservice_api as appservice;
    #[cfg(any(feature = "client-api-c", feature = "client-api-s"))]
    #[doc(inline)]
    pub use ruma_client_api as client;
    // The metadata macro is `#[doc(hidden)]` by default to only show it in the `api` module
    // instead of at the root of `ruma_common`, so we need to explicitly inline it where we
    // want it.
    #[doc(inline)]
    pub use ruma_common::api::metadata;
    pub use ruma_common::api::*;
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

/// Canonical JSON types and related functions.
pub mod canonical_json {
    // The assert_to_canonical_json_eq macro is `#[doc(hidden)]` by default to only show it in the
    // `canonical_json` module instead of at the root of `ruma_common`, so we need to explicitly
    // inline it where we want it.
    #[doc(inline)]
    pub use ruma_common::canonical_json::assert_to_canonical_json_eq;
    pub use ruma_common::canonical_json::*;
}

#[doc(no_inline)]
pub use assign::assign;
#[doc(no_inline)]
pub use js_int::{Int, UInt, int, uint};
#[doc(no_inline)]
pub use js_option::JsOption;
#[cfg(all(feature = "events", feature = "unstable-msc4334"))]
#[doc(no_inline)]
pub use language_tags::LanguageTag;
pub use ruma_common::{CanonicalJsonError, CanonicalJsonObject, CanonicalJsonValue, *};
pub use web_time as time;
