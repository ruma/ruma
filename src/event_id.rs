//! Matrix event identifiers.

use std::{borrow::Cow, convert::TryFrom, num::NonZeroU8};

#[cfg(feature = "diesel")]
use diesel::sql_types::Text;

use crate::{error::Error, generate_localpart, parse_id, validate_id};

/// A Matrix event ID.
///
/// An `EventId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// # Room versions
///
/// Matrix specifies multiple [room versions](https://matrix.org/docs/spec/#room-versions) and the
/// format of event identifiers differ between them. The original format used by room versions 1
/// and 2 uses a short pseudorandom "localpart" followed by the hostname and port of the
/// originating homeserver. Later room versions change event identifiers to be a hash of the event
/// encoded with Base64. Some of the methods provided by `EventId` are only relevant to the
/// original event format.
///
/// ```
/// # use std::convert::TryFrom;
/// # use ruma_identifiers::EventId;
/// // Original format
/// assert_eq!(
///     EventId::try_from("$h29iv0s8:example.com").unwrap().as_ref(),
///     "$h29iv0s8:example.com"
/// );
/// // Room version 3 format
/// assert_eq!(
///     EventId::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk").unwrap().as_ref(),
///     "$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk"
/// );
/// // Room version 4 format
/// assert_eq!(
///     EventId::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg").unwrap().as_ref(),
///     "$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg"
/// );
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "diesel", derive(FromSqlRow, QueryId, AsExpression, SqlType))]
#[cfg_attr(feature = "diesel", sql_type = "Text")]
pub struct EventId {
    full_id: String,
    colon_idx: Option<NonZeroU8>,
}

impl EventId {
    /// Attempts to generate an `EventId` for the given origin server with a localpart consisting
    /// of 18 random ASCII characters. This should only be used for events in the original format
    /// as used by Matrix room versions 1 and 2.
    ///
    /// Does not currently ever fail, but may fail in the future if the homeserver cannot be parsed
    /// parsed as a valid host.
    pub fn new(homeserver_host: &str) -> Result<Self, Error> {
        let full_id = format!("${}:{}", generate_localpart(18), homeserver_host);

        Ok(Self {
            full_id,
            colon_idx: NonZeroU8::new(19),
        })
    }

    /// Returns the host of the event ID, containing the server name (including the port) of the
    /// originating homeserver. Only applicable to events in the original format as used by Matrix
    /// room versions 1 and 2.
    pub fn hostname(&self) -> Option<&str> {
        self.colon_idx
            .map(|idx| &self.full_id[idx.get() as usize + 1..])
    }

    /// Returns the event's unique ID. For the original event format as used by Matrix room
    /// versions 1 and 2, this is the "localpart" that precedes the homeserver. For later formats,
    /// this is the entire ID without the leading $ sigil.
    pub fn localpart(&self) -> &str {
        let idx = match self.colon_idx {
            Some(idx) => idx.get() as usize,
            None => self.full_id.len(),
        };

        &self.full_id[1..idx]
    }
}

impl TryFrom<Cow<'_, str>> for EventId {
    type Error = Error;

    /// Attempts to create a new Matrix event ID from a string representation.
    ///
    /// If using the original event format as used by Matrix room versions 1 and 2, the string must
    /// include the leading $ sigil, the localpart, a literal colon, and a valid homeserver
    /// hostname.
    fn try_from(event_id: Cow<'_, str>) -> Result<Self, Self::Error> {
        if event_id.contains(':') {
            let colon_idx = parse_id(&event_id, &['$'])?;

            Ok(Self {
                full_id: event_id.into_owned(),
                colon_idx: Some(colon_idx),
            })
        } else {
            validate_id(&event_id, &['$'])?;

            Ok(Self {
                full_id: event_id.into_owned(),
                colon_idx: None,
            })
        }
    }
}

common_impls!(EventId, "a Matrix event ID");

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use serde_json::{from_str, to_string};

    use super::EventId;
    use crate::error::Error;

    #[test]
    fn valid_original_event_id() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com")
                .expect("Failed to create EventId.")
                .as_ref(),
            "$39hvsi03hlne:example.com"
        );
    }

    #[test]
    fn valid_base64_event_id() {
        assert_eq!(
            EventId::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                .expect("Failed to create EventId.")
                .as_ref(),
            "$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk"
        )
    }

    #[test]
    fn valid_url_safe_base64_event_id() {
        assert_eq!(
            EventId::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                .expect("Failed to create EventId.")
                .as_ref(),
            "$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg"
        )
    }

    #[test]
    fn generate_random_valid_event_id() {
        let event_id = EventId::new("example.com").expect("Failed to generate EventId.");
        let id_str: &str = event_id.as_ref();

        assert!(id_str.starts_with('$'));
        assert_eq!(id_str.len(), 31);
    }

    /*#[test]
    fn generate_random_invalid_event_id() {
        assert!(EventId::new("").is_err());
    }*/

    #[test]
    fn serialize_valid_original_event_id() {
        assert_eq!(
            to_string(
                &EventId::try_from("$39hvsi03hlne:example.com").expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$39hvsi03hlne:example.com""#
        );
    }

    #[test]
    fn serialize_valid_base64_event_id() {
        assert_eq!(
            to_string(
                &EventId::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                    .expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk""#
        );
    }

    #[test]
    fn serialize_valid_url_safe_base64_event_id() {
        assert_eq!(
            to_string(
                &EventId::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                    .expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg""#
        );
    }

    #[test]
    fn deserialize_valid_original_event_id() {
        assert_eq!(
            from_str::<EventId>(r#""$39hvsi03hlne:example.com""#)
                .expect("Failed to convert JSON to EventId"),
            EventId::try_from("$39hvsi03hlne:example.com").expect("Failed to create EventId.")
        );
    }

    #[test]
    fn deserialize_valid_base64_event_id() {
        assert_eq!(
            from_str::<EventId>(r#""$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk""#)
                .expect("Failed to convert JSON to EventId"),
            EventId::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                .expect("Failed to create EventId.")
        );
    }

    #[test]
    fn deserialize_valid_url_safe_base64_event_id() {
        assert_eq!(
            from_str::<EventId>(r#""$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg""#)
                .expect("Failed to convert JSON to EventId"),
            EventId::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                .expect("Failed to create EventId.")
        );
    }

    #[test]
    fn valid_original_event_id_with_explicit_standard_port() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com:443")
                .expect("Failed to create EventId.")
                .as_ref(),
            "$39hvsi03hlne:example.com:443"
        );
    }

    #[test]
    fn valid_original_event_id_with_non_standard_port() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com:5000")
                .expect("Failed to create EventId.")
                .as_ref(),
            "$39hvsi03hlne:example.com:5000"
        );
    }

    #[test]
    fn missing_original_event_id_sigil() {
        assert_eq!(
            EventId::try_from("39hvsi03hlne:example.com").unwrap_err(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_base64_event_id_sigil() {
        assert_eq!(
            EventId::try_from("acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk").unwrap_err(),
            Error::MissingSigil
        );
    }

    #[test]
    fn missing_url_safe_base64_event_id_sigil() {
        assert_eq!(
            EventId::try_from("Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg").unwrap_err(),
            Error::MissingSigil
        );
    }

    /*#[test]
    fn invalid_event_id_host() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:/").unwrap_err(),
            Error::InvalidHost
        );
    }

    #[test]
    fn invalid_event_id_port() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com:notaport").unwrap_err(),
            Error::InvalidHost
        );
    }*/
}
