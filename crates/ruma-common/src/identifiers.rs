//! Types for [Matrix](https://matrix.org/) identifiers for devices, events, keys, rooms, servers,
//! users and URIs.

// FIXME: Remove once lint doesn't trigger on std::convert::TryFrom in identifiers/macros.rs anymore
#![allow(unused_qualifications)]

use std::convert::TryFrom;

use serde::de::{self, Deserializer, Unexpected};

#[doc(inline)]
pub use self::{
    client_secret::ClientSecret,
    crypto_algorithms::{DeviceKeyAlgorithm, EventEncryptionAlgorithm, SigningKeyAlgorithm},
    device_id::DeviceId,
    device_key_id::DeviceKeyId,
    event_id::EventId,
    key_id::{DeviceSigningKeyId, KeyId, ServerSigningKeyId, SigningKeyId},
    key_name::KeyName,
    matrix_uri::{MatrixToUri, MatrixUri},
    mxc_uri::MxcUri,
    room_alias_id::RoomAliasId,
    room_id::RoomId,
    room_name::RoomName,
    room_or_room_alias_id::RoomOrAliasId,
    room_version_id::RoomVersionId,
    server_name::ServerName,
    session_id::SessionId,
    signatures::{DeviceSignatures, EntitySignatures, ServerSignatures, Signatures},
    transaction_id::TransactionId,
    user_id::UserId,
};
#[doc(inline)]
pub use ruma_identifiers_validation::error::Error;

#[macro_use]
mod macros;

pub mod matrix_uri;
pub mod user_id;

mod client_secret;
mod crypto_algorithms;
mod device_id;
mod device_key_id;
mod event_id;
mod key_id;
mod key_name;
mod mxc_uri;
mod room_alias_id;
mod room_id;
mod room_name;
mod room_or_room_alias_id;
mod room_version_id;
mod server_name;
mod session_id;
mod signatures;
mod transaction_id;

/// Generates a random identifier localpart.
#[cfg(feature = "rand")]
fn generate_localpart(length: usize) -> Box<str> {
    use rand::Rng as _;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .map(char::from)
        .take(length)
        .collect::<String>()
        .into_boxed_str()
}

/// Deserializes any type of id using the provided `TryFrom` implementation.
///
/// This is a helper function to reduce the boilerplate of the `Deserialize` implementations.
fn deserialize_id<'de, D, T>(deserializer: D, expected_str: &str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: for<'a> TryFrom<&'a str>,
{
    ruma_serde::deserialize_cow_str(deserializer).and_then(|v| {
        T::try_from(&v).map_err(|_| de::Error::invalid_value(Unexpected::Str(&v), &expected_str))
    })
}

/// Shorthand for `<&DeviceId>::from`.
#[macro_export]
macro_rules! device_id {
    ($s:expr) => {
        <&$crate::DeviceId as ::std::convert::From<_>>::from($s)
    };
}

// A plain re-export shows up in rustdoc despite doc(hidden). Use a module instead.
// Bug report: https://github.com/rust-lang/rust/issues/83939
#[doc(hidden)]
pub mod _macros {
    pub use ruma_macros::{
        device_key_id, event_id, mxc_uri, room_alias_id, room_id, room_version_id, server_name,
        server_signing_key_id, user_id,
    };
}

/// Compile-time checked `DeviceKeyId` construction.
#[macro_export]
macro_rules! device_key_id {
    ($s:literal) => {
        $crate::_macros::device_key_id!($crate, $s)
    };
}

/// Compile-time checked `EventId` construction.
#[macro_export]
macro_rules! event_id {
    ($s:literal) => {
        $crate::_macros::event_id!($crate, $s)
    };
}

/// Compile-time checked `RoomAliasId` construction.
#[macro_export]
macro_rules! room_alias_id {
    ($s:literal) => {
        $crate::_macros::room_alias_id!($crate, $s)
    };
}

/// Compile-time checked `RoomId` construction.
#[macro_export]
macro_rules! room_id {
    ($s:literal) => {
        $crate::_macros::room_id!($crate, $s)
    };
}

/// Compile-time checked `RoomVersionId` construction.
#[macro_export]
macro_rules! room_version_id {
    ($s:literal) => {
        $crate::_macros::room_version_id!($crate, $s)
    };
}

/// Compile-time checked `ServerSigningKeyId` construction.
#[macro_export]
macro_rules! server_signing_key_id {
    ($s:literal) => {
        $crate::_macros::server_signing_key_id!($crate, $s)
    };
}

/// Compile-time checked `ServerName` construction.
#[macro_export]
macro_rules! server_name {
    ($s:literal) => {
        $crate::_macros::server_name!($crate, $s)
    };
}

/// Compile-time checked `MxcUri` construction.
#[macro_export]
macro_rules! mxc_uri {
    ($s:literal) => {
        $crate::_macros::mxc_uri!($crate, $s)
    };
}

/// Compile-time checked `UserId` construction.
#[macro_export]
macro_rules! user_id {
    ($s:literal) => {
        $crate::_macros::user_id!($crate, $s)
    };
}
