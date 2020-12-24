//! Matrix room version identifiers.

use std::{cmp::Ordering, convert::TryFrom, str::FromStr};

use ruma_serde_macros::DisplayAsRefStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::Error;

/// A Matrix room version ID.
///
/// A `RoomVersionId` can be or converted or deserialized from a string slice, and can be converted
/// or serialized back into a string as needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomVersionId;
/// assert_eq!(RoomVersionId::try_from("1").unwrap().as_ref(), "1");
/// ```
///
/// Any string consisting of at minimum 1, at maximum 32 unicode codepoints is a room version ID.
/// Custom room versions or ones that were introduced into the specification after this code was
/// written are represented by a hidden enum variant. You can still construct them the same, and
/// check for them using one of `RoomVersionId`s `PartialEq` implementations or through `.as_str()`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, DisplayAsRefStr)]
pub enum RoomVersionId {
    /// A version 1 room.
    Version1,

    /// A version 2 room.
    Version2,

    /// A version 3 room.
    Version3,

    /// A version 4 room.
    Version4,

    /// A version 5 room.
    Version5,

    /// A version 6 room.
    Version6,

    #[doc(hidden)]
    _Custom(CustomRoomVersion),
}

impl RoomVersionId {
    /// Creates a string slice from this `RoomVersionId`.
    pub fn as_str(&self) -> &str {
        // FIXME: Add support for non-`str`-deref'ing types for fallback to AsRefStr derive and
        //        implement this function in terms of `AsRef<str>`
        match &self {
            Self::Version1 => "1",
            Self::Version2 => "2",
            Self::Version3 => "3",
            Self::Version4 => "4",
            Self::Version5 => "5",
            Self::Version6 => "6",
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
            RoomVersionId::Version1 => "1".to_owned(),
            RoomVersionId::Version2 => "2".to_owned(),
            RoomVersionId::Version3 => "3".to_owned(),
            RoomVersionId::Version4 => "4".to_owned(),
            RoomVersionId::Version5 => "5".to_owned(),
            RoomVersionId::Version6 => "6".to_owned(),
            RoomVersionId::_Custom(version) => version.into(),
        }
    }
}

impl AsRef<str> for RoomVersionId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialOrd for RoomVersionId {
    /// Compare the two given room version IDs by comparing their string representations.
    ///
    /// Please be aware that room version IDs don't have a defined ordering in the Matrix
    /// specification. This implementation only exists to be able to use `RoomVersionId`s or
    /// types containing `RoomVersionId`s as `BTreeMap` keys.
    fn partial_cmp(&self, other: &RoomVersionId) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl Ord for RoomVersionId {
    /// Compare the two given room version IDs by comparing their string representations.
    ///
    /// Please be aware that room version IDs don't have a defined ordering in the Matrix
    /// specification. This implementation only exists to be able to use `RoomVersionId`s or
    /// types containing `RoomVersionId`s as `BTreeMap` keys.
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

#[cfg(feature = "serde")]
impl Serialize for RoomVersionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for RoomVersionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        crate::deserialize_id(deserializer, "a Matrix room version ID as a string")
    }
}

/// Attempts to create a new Matrix room version ID from a string representation.
fn try_from<S>(room_version_id: S) -> Result<RoomVersionId, Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let version = match room_version_id.as_ref() {
        "1" => RoomVersionId::Version1,
        "2" => RoomVersionId::Version2,
        "3" => RoomVersionId::Version3,
        "4" => RoomVersionId::Version4,
        "5" => RoomVersionId::Version5,
        "6" => RoomVersionId::Version6,
        custom => {
            ruma_identifiers_validation::room_version_id::validate(custom)?;
            RoomVersionId::_Custom(CustomRoomVersion(room_version_id.into()))
        }
    };

    Ok(version)
}

impl FromStr for RoomVersionId {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        try_from(s)
    }
}

impl TryFrom<&str> for RoomVersionId {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Error> {
        try_from(s)
    }
}

impl TryFrom<String> for RoomVersionId {
    type Error = crate::Error;

    fn try_from(s: String) -> Result<Self, Error> {
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
    use std::convert::TryFrom;

    #[cfg(feature = "serde")]
    use serde_json::{from_str, to_string};

    use super::RoomVersionId;
    use crate::Error;

    #[test]
    fn valid_version_1_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("1").expect("Failed to create RoomVersionId.").as_ref(),
            "1"
        );
    }

    #[test]
    fn valid_version_2_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("2").expect("Failed to create RoomVersionId.").as_ref(),
            "2"
        );
    }

    #[test]
    fn valid_version_3_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("3").expect("Failed to create RoomVersionId.").as_ref(),
            "3"
        );
    }

    #[test]
    fn valid_version_4_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("4").expect("Failed to create RoomVersionId.").as_ref(),
            "4"
        );
    }

    #[test]
    fn valid_version_5_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("5").expect("Failed to create RoomVersionId.").as_ref(),
            "5"
        );
    }

    #[test]
    fn valid_version_6_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("6").expect("Failed to create RoomVersionId.").as_ref(),
            "6"
        );
    }

    #[test]
    fn valid_custom_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("io.ruma.1").expect("Failed to create RoomVersionId.").as_ref(),
            "io.ruma.1"
        );
    }

    #[test]
    fn empty_room_version_id() {
        assert_eq!(RoomVersionId::try_from(""), Err(Error::EmptyRoomVersionId));
    }

    #[test]
    fn over_max_code_point_room_version_id() {
        assert_eq!(
            RoomVersionId::try_from("0123456789012345678901234567890123456789"),
            Err(Error::MaximumLengthExceeded)
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_official_room_id() {
        assert_eq!(
            to_string(&RoomVersionId::try_from("1").expect("Failed to create RoomVersionId."))
                .expect("Failed to convert RoomVersionId to JSON."),
            r#""1""#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_official_room_id() {
        let deserialized =
            from_str::<RoomVersionId>(r#""1""#).expect("Failed to convert RoomVersionId to JSON.");

        assert_eq!(deserialized, RoomVersionId::Version1);

        assert_eq!(
            deserialized,
            RoomVersionId::try_from("1").expect("Failed to create RoomVersionId.")
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_custom_room_id() {
        assert_eq!(
            to_string(
                &RoomVersionId::try_from("io.ruma.1").expect("Failed to create RoomVersionId.")
            )
            .expect("Failed to convert RoomVersionId to JSON."),
            r#""io.ruma.1""#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_custom_room_id() {
        let deserialized = from_str::<RoomVersionId>(r#""io.ruma.1""#)
            .expect("Failed to convert RoomVersionId to JSON.");

        assert_eq!(
            deserialized,
            RoomVersionId::try_from("io.ruma.1").expect("Failed to create RoomVersionId.")
        );
    }
}
