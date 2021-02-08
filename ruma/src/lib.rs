//! Types and traits for working with the Matrix protocol.
//!
//! This crate re-exports things from all of the other ruma crates so you don't
//! have to manually keep all the versions in sync.
//!
//! Which crates are re-exported can be configured through cargo features.
//! Depending on which parts of Matrix are relevant to you, activate the
//! following features:
//!
//! * `client-api` for the client-server API
//! * `federation-api` for the server-server (federation) API
//! * `appservice-api` for the application service API
//!
//! There's also the features `api`, `events` and `signatures` for the
//! submodules of the same names. Usually they are activated by one of the
//! other features when needed. If you are viewing this on `docs.rs`, you can
//! have a look at the feature dependencies by clicking 'Feature flags' in the
//! toolbar at the top.

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(no_inline)]
pub use assign::assign;
#[doc(no_inline)]
pub use js_int::{int, uint, Int, UInt};

pub use ruma_common::*;
#[doc(inline)]
pub use ruma_identifiers as identifiers;
#[doc(inline)]
pub use ruma_serde as serde;

pub use ruma_serde::Outgoing;

#[allow(deprecated)] // Allow re-export of deprecated items
pub use ruma_identifiers::{
    device_id, device_key_id, event_id, room_alias_id, room_id, room_version_id, server_key_id,
    server_name, server_signing_key_id, user_id, DeviceId, DeviceKeyAlgorithm, DeviceKeyId,
    EventId, RoomAliasId, RoomId, RoomIdOrAliasId, RoomVersionId, ServerName, ServerSigningKeyId,
    SigningKeyAlgorithm, UserId,
};

#[cfg(feature = "events")]
#[cfg_attr(docsrs, doc(cfg(feature = "events")))]
#[doc(inline)]
pub use ruma_events as events;
#[cfg(feature = "signatures")]
#[cfg_attr(docsrs, doc(cfg(feature = "signatures")))]
#[doc(inline)]
pub use ruma_signatures as signatures;

/// Rust types for various Matrix APIs requests and responses and abstractions for them.
#[cfg(feature = "api")]
#[cfg_attr(docsrs, doc(cfg(feature = "api")))]
pub mod api {
    pub use ruma_api::*;

    #[cfg(feature = "appservice-api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "appservice-api")))]
    #[doc(inline)]
    pub use ruma_appservice_api as appservice;
    #[cfg(feature = "client-api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "client-api")))]
    #[doc(inline)]
    pub use ruma_client_api as client;
    #[cfg(feature = "federation-api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "federation-api")))]
    #[doc(inline)]
    pub use ruma_federation_api as federation;
    #[cfg(feature = "identity-service-api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "identity-service-api")))]
    #[doc(inline)]
    pub use ruma_identity_service_api as identity_service;
    #[cfg(feature = "push-gateway-api")]
    #[cfg_attr(docsrs, doc(cfg(feature = "push-gateway-api")))]
    #[doc(inline)]
    pub use ruma_push_gateway_api as push_gateway;
}
