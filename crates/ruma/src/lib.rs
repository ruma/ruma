#![doc(html_favicon_url = "https://www.ruma.io/favicon.ico")]
#![doc(html_logo_url = "https://www.ruma.io/images/logo.png")]
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
//! * `either`
//! * `rand`
//! * `markdown`
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
//! * `unstable-pre-spec` -- Undocumented Matrix features that may be subject to change or removal.
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
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[doc(inline)]
pub use ruma_common::serde;

#[cfg(feature = "client")]
#[doc(inline)]
pub use ruma_client as client;
#[cfg(feature = "events")]
#[doc(inline)]
pub use ruma_common::events;
#[cfg(feature = "signatures")]
#[doc(inline)]
pub use ruma_signatures as signatures;
#[cfg(feature = "state-res")]
#[doc(inline)]
pub use ruma_state_res as state_res;

/// (De)serializable types for various [Matrix APIs][apis] requests and responses and abstractions
/// for them.
///
/// [apis]: https://spec.matrix.org/v1.2/#matrix-apis
#[cfg(feature = "api")]
pub mod api {
    pub use ruma_common::api::*;

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

#[doc(no_inline)]
pub use assign::assign;
#[doc(no_inline)]
pub use js_int::{int, uint, Int, UInt};
#[cfg(feature = "client-ext-client-api")]
pub use ruma_client::Client;
pub use ruma_common::{
    authentication, device_id, device_key_id, directory, encryption, event_id, exports, matrix_uri,
    mxc_uri, power_levels, presence, push, receipt, room, room_alias_id, room_id, room_version_id,
    serde::Incoming, server_name, server_signing_key_id, thirdparty, to_device, user_id,
    ClientSecret, DeviceId, DeviceKeyAlgorithm, DeviceKeyId, DeviceSignatures, DeviceSigningKeyId,
    EntitySignatures, EventEncryptionAlgorithm, EventId, IdParseError, KeyId, KeyName, MatrixToUri,
    MatrixUri, MilliSecondsSinceUnixEpoch, MxcUri, OwnedClientSecret, OwnedDeviceId,
    OwnedDeviceKeyId, OwnedDeviceSigningKeyId, OwnedEventId, OwnedKeyId, OwnedKeyName, OwnedMxcUri,
    OwnedRoomAliasId, OwnedRoomId, OwnedRoomName, OwnedRoomOrAliasId, OwnedServerName,
    OwnedServerSigningKeyId, OwnedSessionId, OwnedSigningKeyId, OwnedTransactionId, OwnedUserId,
    PrivOwnedStr, RoomAliasId, RoomId, RoomName, RoomOrAliasId, RoomVersionId,
    SecondsSinceUnixEpoch, ServerName, ServerSignatures, ServerSigningKeyId, SessionId, Signatures,
    SigningKeyAlgorithm, TransactionId, UserId,
};
