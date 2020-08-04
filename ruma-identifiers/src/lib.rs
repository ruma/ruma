//! Crate **ruma_identifiers** contains types for [Matrix](https://matrix.org/) identifiers
//! for events, rooms, room aliases, room versions, and users.

#![warn(
    rust_2018_idioms,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::convert::TryFrom;

#[cfg(feature = "serde")]
use serde::de::{self, Deserialize as _, Deserializer, Unexpected};

#[doc(inline)]
pub use crate::{
    device_id::DeviceId, device_key_id::DeviceKeyId, event_id::EventId, room_alias_id::RoomAliasId,
    room_id::RoomId, room_id_or_room_alias_id::RoomIdOrAliasId, room_version_id::RoomVersionId,
    server_key_id::ServerKeyId, server_name::ServerName, user_id::UserId,
};
#[doc(inline)]
pub use ruma_identifiers_validation::{
    error::Error,
    key_algorithms::{DeviceKeyAlgorithm, ServerKeyAlgorithm},
};

#[macro_use]
mod macros;

pub mod device_id;
pub mod user_id;

mod device_key_id;
mod event_id;
mod room_alias_id;
mod room_id;
mod room_id_or_room_alias_id;
mod room_version_id;
mod server_key_id;
mod server_name;

/// Check whether a given string is a valid server name according to [the specification][].
///
/// [the specification]: https://matrix.org/docs/spec/appendices#server-name
#[deprecated = "Use the [`ServerName`](server_name/struct.ServerName.html) type instead."]
pub fn is_valid_server_name(name: &str) -> bool {
    <&ServerName>::try_from(name).is_ok()
}

/// Generates a random identifier localpart.
#[cfg(feature = "rand")]
fn generate_localpart(length: usize) -> Box<str> {
    use rand::Rng as _;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(length)
        .collect::<String>()
        .into_boxed_str()
}

/// Deserializes any type of id using the provided TryFrom implementation.
///
/// This is a helper function to reduce the boilerplate of the Deserialize implementations.
#[cfg(feature = "serde")]
fn deserialize_id<'de, D, T>(deserializer: D, expected_str: &str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: for<'a> std::convert::TryFrom<&'a str>,
{
    std::borrow::Cow::<'_, str>::deserialize(deserializer).and_then(|v| {
        T::try_from(&v).map_err(|_| de::Error::invalid_value(Unexpected::Str(&v), &expected_str))
    })
}

/// Shorthand for `Box::<DeviceId>::from`.
#[macro_export]
macro_rules! device_id {
    ($s:tt) => {
        ::std::boxed::Box<$crate::DeviceId>::from($s)
    };
}

#[doc(hidden)]
pub use ruma_identifiers_macros as _macros;

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

/// Compile-time checked `ServerKeyId` construction.
#[macro_export]
macro_rules! server_key_id {
    ($s:literal) => {
        $crate::_macros::server_key_id!($crate, $s)
    };
}

/// Compile-time checked `ServerName` construction.
#[macro_export]
macro_rules! server_name {
    ($s:literal) => {
        $crate::_macros::server_name!($crate, $s)
    };
}

/// Compile-time checked `UserId` construction.
#[macro_export]
macro_rules! user_id {
    ($s:literal) => {
        $crate::_macros::user_id!($crate, $s)
    };
}
