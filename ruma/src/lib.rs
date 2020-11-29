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

#![deny(missing_docs)]

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

pub use ruma_identifiers::{
    device_id, device_key_id, event_id, room_alias_id, room_id, room_version_id, server_key_id,
    server_name, user_id, DeviceId, DeviceKeyAlgorithm, DeviceKeyId, EventId, RoomAliasId, RoomId,
    RoomIdOrAliasId, RoomVersionId, ServerName, ServerSigningKeyId, SigningKeyAlgorithm, UserId,
};

#[cfg(feature = "events")]
#[doc(inline)]
pub use ruma_events as events;
#[cfg(feature = "signatures")]
#[doc(inline)]
pub use ruma_signatures as signatures;

/// Rust types for various Matrix APIs requests and responses and abstractions for them.
#[cfg(feature = "ruma-api")]
#[doc(inline)]
pub mod api {
    pub use ruma_api::*;

    #[cfg(feature = "appservice-api")]
    #[doc(inline)]
    pub use ruma_appservice_api as appservice;
    #[cfg(feature = "client-api")]
    #[doc(inline)]
    pub use ruma_client_api as client;
    #[cfg(feature = "federation-api")]
    #[doc(inline)]
    pub use ruma_federation_api as federation;
    #[cfg(feature = "identity-service-api")]
    #[doc(inline)]
    pub use ruma_identity_service_api as identity_service;
    #[cfg(feature = "push-gateway-api")]
    #[doc(inline)]
    pub use ruma_push_gateway_api as push_gateway;
}
