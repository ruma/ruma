//! Crate **ruma_identifiers** contains types for [Matrix](https://matrix.org/) identifiers
//! for events, rooms, room aliases, room versions, and users.

#![warn(
    rust_2018_idioms,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]
// Since we support Rust 1.36.0, we can't apply this suggestion yet
#![allow(clippy::use_self)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{convert::TryFrom, num::NonZeroU8};

#[cfg(feature = "serde")]
use serde::de::{self, Deserialize as _, Deserializer, Unexpected};

#[doc(inline)]
pub use crate::{
    device_id::DeviceId,
    device_key_id::DeviceKeyId,
    error::Error,
    event_id::EventId,
    key_algorithms::{DeviceKeyAlgorithm, ServerKeyAlgorithm},
    room_alias_id::RoomAliasId,
    room_id::RoomId,
    room_id_or_room_alias_id::RoomIdOrAliasId,
    room_version_id::RoomVersionId,
    server_key_id::ServerKeyId,
    server_name::ServerName,
    user_id::UserId,
};

#[macro_use]
mod macros;

pub mod device_id;
pub mod user_id;

mod device_key_id;
mod error;
mod event_id;
mod key_algorithms;
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

/// All identifiers must be 255 bytes or less.
const MAX_BYTES: usize = 255;
/// The minimum number of characters an ID can be.
///
/// This is an optimization and not required by the spec. The shortest possible valid ID is a sigil
/// + a single character local ID + a colon + a single character hostname.
const MIN_CHARS: usize = 4;

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

/// Checks if an identifier is valid.
fn validate_id(id: &str, valid_sigils: &[char]) -> Result<(), Error> {
    if id.len() > MAX_BYTES {
        return Err(Error::MaximumLengthExceeded);
    }

    if id.len() < MIN_CHARS {
        return Err(Error::MinimumLengthNotSatisfied);
    }

    if !valid_sigils.contains(&id.chars().next().unwrap()) {
        return Err(Error::MissingSigil);
    }

    Ok(())
}

/// Checks an identifier that contains a localpart and hostname for validity,
/// and returns the index of the colon that separates the two.
fn parse_id(id: &str, valid_sigils: &[char]) -> Result<NonZeroU8, Error> {
    validate_id(id, valid_sigils)?;

    let colon_idx = id.find(':').ok_or(Error::MissingDelimiter)?;
    if colon_idx < 2 {
        return Err(Error::InvalidLocalPart);
    }

    server_name::validate(&id[colon_idx + 1..])?;

    Ok(NonZeroU8::new(colon_idx as u8).unwrap())
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
