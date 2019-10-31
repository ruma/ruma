//! Crate **ruma_identifiers** contains types for [Matrix](https://matrix.org/) identifiers
//! for events, rooms, room aliases, room versions, and users.

#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]
// Since we support Rust 1.34.2, we can't apply this suggestion yet
#![allow(clippy::use_self)]

#[cfg(feature = "diesel")]
#[cfg_attr(feature = "diesel", macro_use)]
extern crate diesel;

use std::fmt::{Formatter, Result as FmtResult};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use url::Url;

pub use url::Host;

#[doc(inline)]
pub use crate::device_id::DeviceId;
pub use crate::{
    error::Error, event_id::EventId, room_alias_id::RoomAliasId, room_id::RoomId,
    room_id_or_room_alias_id::RoomIdOrAliasId, room_version_id::RoomVersionId, user_id::UserId,
};

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
/// The number of bytes in a valid sigil.
const SIGIL_BYTES: usize = 1;

/// `Display` implementation shared by identifier types.
fn display(
    f: &mut Formatter<'_>,
    sigil: char,
    localpart: &str,
    hostname: &Host,
    port: u16,
) -> FmtResult {
    if port == 443 {
        write!(f, "{}{}:{}", sigil, localpart, hostname)
    } else {
        write!(f, "{}{}:{}:{}", sigil, localpart, hostname, port)
    }
}

/// Generates a random identifier localpart.
fn generate_localpart(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect()
}

/// Checks if an identifier is within the acceptable byte lengths.
fn validate_id(id: &str) -> Result<(), Error> {
    if id.len() > MAX_BYTES {
        return Err(Error::MaximumLengthExceeded);
    }

    if id.len() < MIN_CHARS {
        return Err(Error::MinimumLengthNotSatisfied);
    }

    Ok(())
}

/// Parses the localpart, host, and port from a string identifier.
fn parse_id(required_sigil: char, id: &str) -> Result<(&str, Host, u16), Error> {
    validate_id(id)?;

    if !id.starts_with(required_sigil) {
        return Err(Error::MissingSigil);
    }

    let delimiter_index = match id.find(':') {
        Some(index) => index,
        None => return Err(Error::MissingDelimiter),
    };

    let localpart = &id[1..delimiter_index];
    let raw_host = &id[delimiter_index + SIGIL_BYTES..];
    let url_string = format!("https://{}", raw_host);
    let url = Url::parse(&url_string)?;

    let host = match url.host() {
        Some(host) => host.to_owned(),
        None => return Err(Error::InvalidHost),
    };

    let port = url.port().unwrap_or(443);

    Ok((localpart, host, port))
}
