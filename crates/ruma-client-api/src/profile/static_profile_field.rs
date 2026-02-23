#![allow(clippy::exhaustive_structs)]

use ruma_common::MxcUri;
use serde::{Serialize, de::DeserializeOwned};

/// Trait implemented by types representing a field in a user's [profile] having a statically-known
/// name.
///
/// [profile]: https://spec.matrix.org/latest/client-server-api/#profiles
pub trait StaticProfileField {
    /// The type for the value of the field.
    type Value: Sized + Serialize + DeserializeOwned;

    /// The string representation of this field.
    const NAME: &str;
}

/// The user's avatar URL.
#[derive(Debug, Clone, Copy)]
pub struct AvatarUrl;

impl StaticProfileField for AvatarUrl {
    type Value = MxcUri;
    const NAME: &str = "avatar_url";
}

/// The user's display name.
#[derive(Debug, Clone, Copy)]
pub struct DisplayName;

impl StaticProfileField for DisplayName {
    type Value = String;
    const NAME: &str = "displayname";
}

/// The user's time zone.
#[derive(Debug, Clone, Copy)]
pub struct TimeZone;

impl StaticProfileField for TimeZone {
    type Value = String;
    const NAME: &str = "m.tz";
}
