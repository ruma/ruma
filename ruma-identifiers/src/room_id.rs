//! Matrix room identifiers.

use std::{convert::TryFrom, fmt, num::NonZeroU8};

use crate::{Error, ServerName};

/// A Matrix room ID.
///
/// A `RoomId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::RoomId;
/// assert_eq!(
///     RoomId::try_from("!n8f893n9:example.com").unwrap().as_ref(),
///     "!n8f893n9:example.com"
/// );
/// ```
#[derive(Clone)]
pub struct RoomId {
    pub(crate) full_id: Box<str>,
    pub(crate) colon_idx: NonZeroU8,
}

impl fmt::Debug for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.full_id)
    }
}

impl RoomId {
    /// Attempts to generate a `RoomId` for the given origin server with a localpart consisting of
    /// 18 random ASCII characters.
    ///
    /// Fails if the given homeserver cannot be parsed as a valid host.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn new(server_name: &ServerName) -> Self {
        use crate::generate_localpart;

        let full_id = format!("!{}:{}", generate_localpart(18), server_name).into();

        Self { full_id, colon_idx: NonZeroU8::new(19).unwrap() }
    }
}

impl RoomId {
    /// Returns the rooms's unique ID.
    pub fn localpart(&self) -> &str {
        &self.full_id[1..self.colon_idx.get() as usize]
    }

    /// Returns the server name of the room ID.
    pub fn server_name(&self) -> &ServerName {
        <&ServerName>::try_from(&self.full_id[self.colon_idx.get() as usize + 1..]).unwrap()
    }
}

/// Attempts to create a new Matrix room ID from a string representation.
///
/// The string must include the leading ! sigil, the localpart, a literal colon, and a server name.
fn try_from<S>(room_id: S) -> Result<RoomId, Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let colon_idx = ruma_identifiers_validation::room_id::validate(room_id.as_ref())?;
    Ok(RoomId { full_id: room_id.into(), colon_idx })
}

common_impls!(RoomId, try_from, "a Matrix room ID");

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    #[cfg(feature = "serde")]
    use serde_json::{from_str, to_string};

    use super::RoomId;
    use crate::Error;

    #[test]
    fn valid_room_id() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com"
        );
    }

    #[test]
    fn empty_localpart() {
        assert_eq!(
            RoomId::try_from("!:example.com").expect("Failed to create RoomId.").as_ref(),
            "!:example.com"
        );
    }

    #[cfg(feature = "rand")]
    #[test]
    fn generate_random_valid_room_id() {
        use crate::ServerName;

        let server_name =
            <&ServerName>::try_from("example.com").expect("Failed to parse ServerName");
        let room_id = RoomId::new(server_name);
        let id_str = room_id.as_str();

        assert!(id_str.starts_with('!'));
        assert_eq!(id_str.len(), 31);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_valid_room_id() {
        assert_eq!(
            to_string(
                &RoomId::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId.")
            )
            .expect("Failed to convert RoomId to JSON."),
            r#""!29fhd83h92h0:example.com""#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_valid_room_id() {
        assert_eq!(
            from_str::<RoomId>(r#""!29fhd83h92h0:example.com""#)
                .expect("Failed to convert JSON to RoomId"),
            RoomId::try_from("!29fhd83h92h0:example.com").expect("Failed to create RoomId.")
        );
    }

    #[test]
    fn valid_room_id_with_explicit_standard_port() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com:443")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com:443"
        );
    }

    #[test]
    fn valid_room_id_with_non_standard_port() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com:5000")
                .expect("Failed to create RoomId.")
                .as_ref(),
            "!29fhd83h92h0:example.com:5000"
        );
    }

    #[test]
    fn missing_room_id_sigil() {
        assert_eq!(RoomId::try_from("carl:example.com").unwrap_err(), Error::MissingLeadingSigil);
    }

    #[test]
    fn missing_room_id_delimiter() {
        assert_eq!(RoomId::try_from("!29fhd83h92h0").unwrap_err(), Error::MissingDelimiter);
    }

    #[test]
    fn invalid_room_id_host() {
        assert_eq!(RoomId::try_from("!29fhd83h92h0:/").unwrap_err(), Error::InvalidServerName);
    }

    #[test]
    fn invalid_room_id_port() {
        assert_eq!(
            RoomId::try_from("!29fhd83h92h0:example.com:notaport").unwrap_err(),
            Error::InvalidServerName
        );
    }
}
