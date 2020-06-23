//! Crate **ruma_identifiers** contains types for [Matrix](https://matrix.org/) identifiers
//! for events, rooms, room aliases, room versions, and users.

#![warn(rust_2018_idioms)]
#![deny(missing_copy_implementations, missing_debug_implementations, missing_docs)]
// Since we support Rust 1.36.0, we can't apply this suggestion yet
#![allow(clippy::use_self)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use core::{convert::TryFrom, num::NonZeroU8};

#[cfg(any(feature = "alloc", feature = "rand", feature = "serde"))]
extern crate alloc;

#[cfg(feature = "serde")]
use serde::de::{self, Deserialize as _, Deserializer, Unexpected};

#[doc(inline)]
pub use crate::error::Error;

#[macro_use]
mod macros;

mod error;

pub mod device_id;
pub mod device_key_id;
pub mod event_id;
pub mod key_algorithms;
pub mod room_alias_id;
pub mod room_id;
pub mod room_id_or_room_alias_id;
pub mod room_version_id;
pub mod server_key_id;
#[allow(deprecated)]
pub mod server_name;
pub mod user_id;

/// Allowed algorithms for homeserver signing keys.
pub type DeviceKeyAlgorithm = key_algorithms::DeviceKeyAlgorithm;

/// An owned device key identifier containing a key algorithm and device ID.
///
/// Can be created via `TryFrom<String>` and `TryFrom<&str>`; implements `Serialize`
/// and `Deserialize` if the `serde` feature is enabled.
#[cfg(feature = "alloc")]
pub type DeviceKeyId = device_key_id::DeviceKeyId<alloc::boxed::Box<str>>;

/// A reference to a device key identifier containing a key algorithm and device ID.
///
/// Can be created via `TryFrom<&str>`; implements `Serialize` and `Deserialize`
/// if the `serde` feature is enabled.
pub type DeviceKeyIdRef<'a> = device_key_id::DeviceKeyId<&'a str>;

/// An owned device ID.
///
/// While this is currently just a `String`, that will likely change in the future.
#[cfg(feature = "alloc")]
pub use device_id::DeviceId;

/// A reference to a device ID.
///
/// While this is currently just a string slice, that will likely change in the future.
pub type DeviceIdRef<'a> = &'a str;

/// An owned event ID.
///
/// Can be created via `new` (if the `rand` feature is enabled) and `TryFrom<String>` +
/// `TryFrom<&str>`, implements `Serialize` and `Deserialize` if the `serde` feature is enabled.
#[cfg(feature = "alloc")]
pub type EventId = event_id::EventId<alloc::boxed::Box<str>>;

/// A reference to an event ID.
///
/// Can be created via `TryFrom<&str>`, implements `Serialize` if the `serde` feature is enabled.
pub type EventIdRef<'a> = event_id::EventId<&'a str>;

/// An owned room alias ID.
///
/// Can be created via `TryFrom<String>` and `TryFrom<&str>`, implements `Serialize` and
/// `Deserialize` if the `serde` feature is enabled.
#[cfg(feature = "alloc")]
pub type RoomAliasId = room_alias_id::RoomAliasId<alloc::boxed::Box<str>>;

/// A reference to a room alias ID.
///
/// Can be created via `TryFrom<&str>`, implements `Serialize` if the `serde` feature is enabled.
pub type RoomAliasIdRef<'a> = room_alias_id::RoomAliasId<&'a str>;

/// An owned room ID.
///
/// Can be created via `new` (if the `rand` feature is enabled) and `TryFrom<String>` +
/// `TryFrom<&str>`, implements `Serialize` and `Deserialize` if the `serde` feature is enabled.
#[cfg(feature = "alloc")]
pub type RoomId = room_id::RoomId<alloc::boxed::Box<str>>;

/// A reference to a room ID.
///
/// Can be created via `TryFrom<&str>`, implements `Serialize` if the `serde` feature is enabled.
pub type RoomIdRef<'a> = room_id::RoomId<&'a str>;

/// An owned room alias ID or room ID.
///
/// Can be created via `TryFrom<String>`, `TryFrom<&str>`, `From<RoomId>` and `From<RoomAliasId>`;
/// implements `Serialize` and `Deserialize` if the `serde` feature is enabled.
#[cfg(feature = "alloc")]
pub type RoomIdOrAliasId = room_id_or_room_alias_id::RoomIdOrAliasId<alloc::boxed::Box<str>>;

/// A reference to a room alias ID or room ID.
///
/// Can be created via `TryFrom<&str>`, `From<RoomIdRef>` and `From<RoomAliasIdRef>`; implements
/// `Serialize` if the `serde` feature is enabled.
pub type RoomIdOrAliasIdRef<'a> = room_id_or_room_alias_id::RoomIdOrAliasId<&'a str>;

/// An owned room version ID.
///
/// Can be created using the `version_N` constructor functions, `TryFrom<String>` and
/// `TryFrom<&str>`; implements `Serialize` and `Deserialize` if the `serde` feature is enabled.
#[cfg(feature = "alloc")]
pub type RoomVersionId = room_version_id::RoomVersionId<alloc::boxed::Box<str>>;

/// A reference to a room version ID.
///
/// Can be created using the `version_N` constructor functions and via `TryFrom<&str>`, implements
/// `Serialize` if the `serde` feature is enabled.
pub type RoomVersionIdRef<'a> = room_version_id::RoomVersionId<&'a str>;

/// Allowed algorithms for homeserver signing keys.
pub type ServerKeyAlgorithm = key_algorithms::ServerKeyAlgorithm;

/// An owned homeserver signing key identifier containing a key algorithm and version.
///
/// Can be created via `TryFrom<String>` and `TryFrom<&str>`; implements `Serialize`
/// and `Deserialize` if the `serde` feature is enabled.
#[cfg(feature = "alloc")]
pub type ServerKeyId = server_key_id::ServerKeyId<alloc::boxed::Box<str>>;

/// A reference to a homeserver signing key identifier containing a key
/// algorithm and version.
///
/// Can be created via `TryFrom<&str>`; implements `Serialize`
/// and `Deserialize` if the `serde` feature is enabled.
pub type ServerKeyIdRef<'a> = server_key_id::ServerKeyId<&'a str>;

/// An owned homeserver IP address or hostname.
///
/// Can be created via `TryFrom<String>` and `TryFrom<&str>`; implements `Serialize`
/// and `Deserialize` if the `serde` feature is enabled.
#[cfg(feature = "alloc")]
pub type ServerName = server_name::ServerName<alloc::boxed::Box<str>>;

/// A reference to a homeserver IP address or hostname.
///
/// Can be created via `TryFrom<&str>`; implements `Serialize`
/// and `Deserialize` if the `serde` feature is enabled.
pub type ServerNameRef<'a> = server_name::ServerName<&'a str>;
/// An owned user ID.
///
/// Can be created via `new` (if the `rand` feature is enabled) and `TryFrom<String>` +
/// `TryFrom<&str>`, implements `Serialize` and `Deserialize` if the `serde` feature is enabled.
#[cfg(feature = "alloc")]
pub type UserId = user_id::UserId<alloc::boxed::Box<str>>;

/// A reference to a user ID.
///
/// Can be created via `TryFrom<&str>`, implements `Serialize` if the `serde` feature is enabled.
pub type UserIdRef<'a> = user_id::UserId<&'a str>;

/// Check whether a given string is a valid server name according to [the specification][].
///
/// [the specification]: https://matrix.org/docs/spec/appendices#server-name
#[deprecated = "Use the [`ServerName`](server_name/struct.ServerName.html) type instead."]
pub fn is_valid_server_name(name: &str) -> bool {
    ServerNameRef::try_from(name).is_ok()
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
fn generate_localpart(length: usize) -> alloc::string::String {
    use rand::Rng as _;
    rand::thread_rng().sample_iter(&rand::distributions::Alphanumeric).take(length).collect()
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

    server_name::ServerName::<&str>::try_from(&id[colon_idx + 1..])?;

    Ok(NonZeroU8::new(colon_idx as u8).unwrap())
}

/// Deserializes any type of id using the provided TryFrom implementation.
///
/// This is a helper function to reduce the boilerplate of the Deserialize implementations.
#[cfg(feature = "serde")]
fn deserialize_id<'de, D, T>(deserializer: D, expected_str: &str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: for<'a> core::convert::TryFrom<&'a str>,
{
    alloc::string::String::deserialize(deserializer).and_then(|v| {
        T::try_from(&v).map_err(|_| de::Error::invalid_value(Unexpected::Str(&v), &expected_str))
    })
}
