//! Matrix identifiers for places where a room ID or room alias ID are used interchangeably.

use std::{
    convert::TryFrom,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[cfg(feature = "diesel")]
use diesel::sql_types::Text;
use serde::{
    de::{Error as SerdeError, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{display, error::Error, room_alias_id::RoomAliasId, room_id::RoomId, validate_id};

/// A Matrix room ID or a Matrix room alias ID.
///
/// `RoomIdOrAliasId` is useful for APIs that accept either kind of room identifier. It is converted
/// from a string slice, and can be converted back into a string as needed. When converted from a
/// string slice, the variant is determined by the leading sigil character.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomIdOrAliasId;
/// assert_eq!(
///     RoomIdOrAliasId::try_from("#ruma:example.com").unwrap().to_string(),
///     "#ruma:example.com"
/// );
///
/// assert_eq!(
///     RoomIdOrAliasId::try_from("!n8f893n9:example.com").unwrap().to_string(),
///     "!n8f893n9:example.com"
/// );
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, QueryId, AsExpression, SqlType))]
#[cfg_attr(feature = "diesel", sql_type = "Text")]
pub enum RoomIdOrAliasId {
    /// A Matrix room alias ID.
    RoomAliasId(RoomAliasId),
    /// A Matrix room ID.
    RoomId(RoomId),
}

/// A serde visitor for `RoomIdOrAliasId`.
struct RoomIdOrAliasIdVisitor;

impl Display for RoomIdOrAliasId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            RoomIdOrAliasId::RoomAliasId(ref room_alias_id) => display(
                f,
                '#',
                room_alias_id.alias(),
                room_alias_id.hostname(),
                room_alias_id.port(),
            ),
            RoomIdOrAliasId::RoomId(ref room_id) => display(
                f,
                '!',
                room_id.opaque_id(),
                room_id.hostname(),
                room_id.port(),
            ),
        }
    }
}

impl Serialize for RoomIdOrAliasId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            RoomIdOrAliasId::RoomAliasId(ref room_alias_id) => {
                serializer.serialize_str(&room_alias_id.to_string())
            }
            RoomIdOrAliasId::RoomId(ref room_id) => serializer.serialize_str(&room_id.to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for RoomIdOrAliasId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(RoomIdOrAliasIdVisitor)
    }
}

impl<'a> TryFrom<&'a str> for RoomIdOrAliasId {
    type Error = Error;

    /// Attempts to create a new Matrix room ID or a room alias ID from a string representation.
    ///
    /// The string must either
    /// include the leading ! sigil, the opaque ID, a literal colon, and a valid server name or
    /// include the leading # sigil, the alias, a literal colon, and a valid server name.
    fn try_from(room_id_or_alias_id: &'a str) -> Result<Self, Error> {
        validate_id(room_id_or_alias_id)?;

        let mut chars = room_id_or_alias_id.chars();

        let sigil = chars.nth(0).expect("ID missing first character.");

        match sigil {
            '#' => {
                let room_alias_id = RoomAliasId::try_from(room_id_or_alias_id)?;
                Ok(RoomIdOrAliasId::RoomAliasId(room_alias_id))
            }
            '!' => {
                let room_id = RoomId::try_from(room_id_or_alias_id)?;
                Ok(RoomIdOrAliasId::RoomId(room_id))
            }
            _ => Err(Error::MissingSigil),
        }
    }
}

impl<'de> Visitor<'de> for RoomIdOrAliasIdVisitor {
    type Value = RoomIdOrAliasId;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "a Matrix room ID or room alias ID as a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: SerdeError,
    {
        match RoomIdOrAliasId::try_from(v) {
            Ok(room_id_or_alias_id) => Ok(room_id_or_alias_id),
            Err(_) => Err(SerdeError::invalid_value(Unexpected::Str(v), &self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use serde_json::{from_str, to_string};

    use super::RoomIdOrAliasId;
    use crate::error::Error;

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            RoomIdOrAliasId::try_from("#ruma:example.com")
                .expect("Failed to create RoomAliasId.")
                .to_string(),
            "#ruma:example.com"
        );
    }

    #[test]
    fn valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            RoomIdOrAliasId::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomId.")
                .to_string(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn missing_sigil_for_room_id_or_alias_id() {
        assert_eq!(
            RoomIdOrAliasId::try_from("ruma:example.com").err().unwrap(),
            Error::MissingSigil
        );
    }

    #[test]
    fn serialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            to_string(
                &RoomIdOrAliasId::try_from("#ruma:example.com")
                    .expect("Failed to create RoomAliasId.")
            )
            .expect("Failed to convert RoomAliasId to JSON."),
            r##""#ruma:example.com""##
        );
    }

    #[test]
    fn serialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            to_string(
                &RoomIdOrAliasId::try_from("!29fhd83h92h0:example.com")
                    .expect("Failed to create RoomId.")
            )
            .expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_alias_id() {
        assert_eq!(
            from_str::<RoomIdOrAliasId>(r##""#ruma:example.com""##)
                .expect("Failed to convert JSON to RoomAliasId"),
            RoomIdOrAliasId::try_from("#ruma:example.com").expect("Failed to create RoomAliasId.")
        );
    }

    #[test]
    fn deserialize_valid_room_id_or_alias_id_with_a_room_id() {
        assert_eq!(
            from_str::<RoomIdOrAliasId>(r##""!29fhd83h92h0:example.com""##)
                .expect("Failed to convert JSON to RoomId"),
            RoomIdOrAliasId::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomAliasId.")
        );
    }
}
