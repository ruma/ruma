//! Matrix room version identifiers.

use std::{cmp::Ordering, str::FromStr};

use ruma_macros::DisplayAsRefStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::IdParseError;

/// A Matrix [room version] ID.
///
/// A `RoomVersionId` can be or converted or deserialized from a string slice, and can be converted
/// or serialized back into a string as needed.
///
/// ```
/// # use ruma_common::RoomVersionId;
/// assert_eq!(RoomVersionId::try_from("1").unwrap().as_str(), "1");
/// ```
///
/// Any string consisting of at minimum 1, at maximum 32 unicode codepoints is a room version ID.
/// Custom room versions or ones that were introduced into the specification after this code was
/// written are represented by a hidden enum variant. You can still construct them the same, and
/// check for them using one of `RoomVersionId`s `PartialEq` implementations or through `.as_str()`.
///
/// [room version]: https://spec.matrix.org/latest/rooms/
#[derive(Clone, Debug, PartialEq, Eq, Hash, DisplayAsRefStr)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum RoomVersionId {
    /// A version 1 room.
    V1,

    /// A version 2 room.
    V2,

    /// A version 3 room.
    V3,

    /// A version 4 room.
    V4,

    /// A version 5 room.
    V5,

    /// A version 6 room.
    V6,

    /// A version 7 room.
    V7,

    /// A version 8 room.
    V8,

    /// A version 9 room.
    V9,

    /// A version 10 room.
    V10,

    /// A version 11 room.
    V11,

    #[doc(hidden)]
    _Custom(CustomRoomVersion),
}

impl RoomVersionId {
    /// Creates a string slice from this `RoomVersionId`.
    pub fn as_str(&self) -> &str {
        // FIXME: Add support for non-`str`-deref'ing types for fallback to AsRefStr derive and
        //        implement this function in terms of `AsRef<str>`
        match &self {
            Self::V1 => "1",
            Self::V2 => "2",
            Self::V3 => "3",
            Self::V4 => "4",
            Self::V5 => "5",
            Self::V6 => "6",
            Self::V7 => "7",
            Self::V8 => "8",
            Self::V9 => "9",
            Self::V10 => "10",
            Self::V11 => "11",
            Self::_Custom(version) => version.as_str(),
        }
    }

    /// Creates a byte slice from this `RoomVersionId`.
    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl From<RoomVersionId> for String {
    fn from(id: RoomVersionId) -> Self {
        match id {
            RoomVersionId::V1 => "1".to_owned(),
            RoomVersionId::V2 => "2".to_owned(),
            RoomVersionId::V3 => "3".to_owned(),
            RoomVersionId::V4 => "4".to_owned(),
            RoomVersionId::V5 => "5".to_owned(),
            RoomVersionId::V6 => "6".to_owned(),
            RoomVersionId::V7 => "7".to_owned(),
            RoomVersionId::V8 => "8".to_owned(),
            RoomVersionId::V9 => "9".to_owned(),
            RoomVersionId::V10 => "10".to_owned(),
            RoomVersionId::V11 => "11".to_owned(),
            RoomVersionId::_Custom(version) => version.into(),
        }
    }
}

impl AsRef<str> for RoomVersionId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for RoomVersionId {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl PartialOrd for RoomVersionId {
    /// Compare the two given room version IDs by comparing their string representations.
    ///
    /// Please be aware that room version IDs don't have a defined ordering in the Matrix
    /// specification. This implementation only exists to be able to use `RoomVersionId`s or
    /// types containing `RoomVersionId`s as `BTreeMap` keys.
    fn partial_cmp(&self, other: &RoomVersionId) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RoomVersionId {
    /// Compare the two given room version IDs by comparing their string representations.
    ///
    /// Please be aware that room version IDs don't have a defined ordering in the Matrix
    /// specification. This implementation only exists to be able to use `RoomVersionId`s or
    /// types containing `RoomVersionId`s as `BTreeMap` keys.
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Serialize for RoomVersionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RoomVersionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_id(deserializer, "a Matrix room version ID as a string")
    }
}

/// Attempts to create a new Matrix room version ID from a string representation.
fn try_from<S>(room_version_id: S) -> Result<RoomVersionId, IdParseError>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let version = match room_version_id.as_ref() {
        "1" => RoomVersionId::V1,
        "2" => RoomVersionId::V2,
        "3" => RoomVersionId::V3,
        "4" => RoomVersionId::V4,
        "5" => RoomVersionId::V5,
        "6" => RoomVersionId::V6,
        "7" => RoomVersionId::V7,
        "8" => RoomVersionId::V8,
        "9" => RoomVersionId::V9,
        "10" => RoomVersionId::V10,
        "11" => RoomVersionId::V11,
        custom => {
            ruma_identifiers_validation::room_version_id::validate(custom)?;
            RoomVersionId::_Custom(CustomRoomVersion(room_version_id.into()))
        }
    };

    Ok(version)
}

impl FromStr for RoomVersionId {
    type Err = IdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        try_from(s)
    }
}

impl TryFrom<&str> for RoomVersionId {
    type Error = IdParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl TryFrom<String> for RoomVersionId {
    type Error = IdParseError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl PartialEq<&str> for RoomVersionId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<RoomVersionId> for &str {
    fn eq(&self, other: &RoomVersionId) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<String> for RoomVersionId {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<RoomVersionId> for String {
    fn eq(&self, other: &RoomVersionId) -> bool {
        self == other.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[doc(hidden)]
#[allow(unknown_lints, unnameable_types)]
pub struct CustomRoomVersion(Box<str>);

#[doc(hidden)]
impl CustomRoomVersion {
    /// Creates a string slice from this `CustomRoomVersion`
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[doc(hidden)]
impl From<CustomRoomVersion> for String {
    fn from(v: CustomRoomVersion) -> Self {
        v.0.into()
    }
}

#[doc(hidden)]
impl AsRef<str> for CustomRoomVersion {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::RoomVersionId;
    use crate::IdParseError;

    #[test]
    fn valid_version_1_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("1").expect("Failed to create RoomVersionId.").as_str(),
            "1"
        );
    }

    #[test]
    fn valid_version_2_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("2").expect("Failed to create RoomVersionId.").as_str(),
            "2"
        );
    }

    #[test]
    fn valid_version_3_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("3").expect("Failed to create RoomVersionId.").as_str(),
            "3"
        );
    }

    #[test]
    fn valid_version_4_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("4").expect("Failed to create RoomVersionId.").as_str(),
            "4"
        );
    }

    #[test]
    fn valid_version_5_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("5").expect("Failed to create RoomVersionId.").as_str(),
            "5"
        );
    }

    #[test]
    fn valid_version_6_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("6").expect("Failed to create RoomVersionId.").as_str(),
            "6"
        );
    }

    #[test]
    fn valid_custom_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("io.ruma.1").expect("Failed to create RoomVersionId.").as_str(),
            "io.ruma.1"
        );
    }

    #[test]
    fn empty_room_version_id() {
        assert_eq!(RoomVersionId::try_from(""), Err(IdParseError::Empty));
    }

    #[test]
    fn over_max_code_point_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("0123456789012345678901234567890123456789"),
            Err(IdParseError::MaximumLengthExceeded)
        );
    }

    #[test]
    fn serialize_official_room_id() {
        assert_eq!(
            serde_json::to_string(
                &RoomVersionId::try_from("1").expect("Failed to create RoomVersionId.")
            )
            .expect("Failed to convert RoomVersionId to JSON."),
            r#""1""#
        );
    }

    #[test]
    fn deserialize_official_room_id() {
        let deserialized = serde_json::from_str::<RoomVersionId>(r#""1""#)
            .expect("Failed to convert RoomVersionId to JSON.");

        assert_eq!(deserialized, RoomVersionId::V1);

        assert_eq!(
            deserialized,
            RoomVersionId::try_from("1").expect("Failed to create RoomVersionId.")
        );
    }

    #[test]
    fn serialize_custom_room_id() {
        assert_eq!(
            serde_json::to_string(
                &RoomVersionId::try_from("io.ruma.1").expect("Failed to create RoomVersionId.")
            )
            .expect("Failed to convert RoomVersionId to JSON."),
            r#""io.ruma.1""#
        );
    }

    #[test]
    fn deserialize_custom_room_id() {
        let deserialized = serde_json::from_str::<RoomVersionId>(r#""io.ruma.1""#)
            .expect("Failed to convert RoomVersionId to JSON.");

        assert_eq!(
            deserialized,
            RoomVersionId::try_from("io.ruma.1").expect("Failed to create RoomVersionId.")
        );
    }

    #[test]
    fn custom_room_id_invalid_character() {
        assert!(serde_json::from_str::<RoomVersionId>(r#""io_ruma_1""#).is_err());
        assert!(serde_json::from_str::<RoomVersionId>(r#""=""#).is_err());
        assert!(serde_json::from_str::<RoomVersionId>(r#""/""#).is_err());
        assert!(serde_json::from_str::<RoomVersionId>(r#"".""#).is_ok());
        assert!(serde_json::from_str::<RoomVersionId>(r#""-""#).is_ok());
        assert_eq!(
            RoomVersionId::try_from("io_ruma_1").unwrap_err(),
            IdParseError::InvalidCharacters
        );
    }
}
