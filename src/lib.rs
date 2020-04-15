//! Crate **ruma_identifiers** contains types for [Matrix](https://matrix.org/) identifiers
//! for events, rooms, room aliases, room versions, and users.

#![warn(rust_2018_idioms)]
#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]
// Since we support Rust 1.36.0, we can't apply this suggestion yet
#![allow(clippy::use_self)]

#[cfg(feature = "diesel")]
#[cfg_attr(feature = "diesel", macro_use)]
extern crate diesel;

use std::{borrow::Cow, convert::TryFrom, num::NonZeroU8};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::de::{self, Deserialize as _, Deserializer, Unexpected};

#[doc(inline)]
pub use crate::device_id::DeviceId;
pub use crate::{
    error::Error, event_id::EventId, room_alias_id::RoomAliasId, room_id::RoomId,
    room_id_or_room_alias_id::RoomIdOrAliasId, room_version_id::RoomVersionId, user_id::UserId,
};

#[macro_use]
mod macros;

pub mod device_id;
#[cfg(feature = "diesel")]
mod diesel_integration;
mod error;
mod event_id;
mod room_alias_id;
mod room_id;
mod room_id_or_room_alias_id;
mod room_version_id;
mod user_id;

/// All identifiers must be 255 bytes or less.
const MAX_BYTES: usize = 255;
/// The minimum number of characters an ID can be.
///
/// This is an optimization and not required by the spec. The shortest possible valid ID is a sigil
/// + a single character local ID + a colon + a single character hostname.
const MIN_CHARS: usize = 4;

/// Generates a random identifier localpart.
fn generate_localpart(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect()
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
    if colon_idx == id.len() - 1 {
        return Err(Error::InvalidHost);
    }

    match NonZeroU8::new(colon_idx as u8) {
        Some(idx) => Ok(idx),
        None => Err(Error::InvalidLocalPart),
    }
}

/// Deserializes any type of id using the provided TryFrom implementation.
///
/// This is a helper function to reduce the boilerplate of the Deserialize implementations.
fn deserialize_id<'de, D, T>(deserializer: D, expected_str: &str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: for<'a> TryFrom<&'a str>,
{
    Cow::<'_, str>::deserialize(deserializer).and_then(|v| {
        T::try_from(&v).map_err(|_| de::Error::invalid_value(Unexpected::Str(&v), &expected_str))
    })
}
