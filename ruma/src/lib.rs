#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
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
    device_id, device_key_id, event_id, mxc_uri, room_alias_id, room_id, room_version_id,
    server_key_id, server_name, server_signing_key_id, user_id, DeviceId, DeviceKeyAlgorithm,
    DeviceKeyId, EventId, MxcUri, RoomAliasId, RoomId, RoomIdOrAliasId, RoomVersionId, ServerName,
    ServerSigningKeyId, SigningKeyAlgorithm, UserId,
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

    #[cfg(feature = "ruma-appservice-api")]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            feature = "appservice-api",
            feature = "appservice-api-c",
            feature = "appservice-api-s"
        )))
    )]
    #[doc(inline)]
    pub use ruma_appservice_api as appservice;
    #[cfg(feature = "ruma-client-api")]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            feature = "client-api",
            feature = "client-api-c",
            feature = "client-api-s"
        )))
    )]
    #[doc(inline)]
    pub use ruma_client_api as client;
    #[cfg(feature = "ruma-federation-api")]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            feature = "federation-api",
            feature = "federation-api-c",
            feature = "federation-api-s"
        )))
    )]
    #[doc(inline)]
    pub use ruma_federation_api as federation;
    #[cfg(feature = "ruma-identity-service-api")]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            feature = "identity-service-api",
            feature = "identity-service-api-c",
            feature = "identity-service-api-s"
        )))
    )]
    #[doc(inline)]
    pub use ruma_identity_service_api as identity_service;
    #[cfg(feature = "ruma-push-gateway-api")]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            feature = "push-gateway-api",
            feature = "push-gateway-api-c",
            feature = "push-gateway-api-s"
        )))
    )]
    #[doc(inline)]
    pub use ruma_push_gateway_api as push_gateway;
}
