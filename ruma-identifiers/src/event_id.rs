//! Matrix event identifiers.

use std::{convert::TryFrom, fmt, num::NonZeroU8};

use crate::{Error, ServerName};

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
#[derive(Clone)]
pub struct EventId {
    full_id: Box<str>,
    colon_idx: Option<NonZeroU8>,
}

impl fmt::Debug for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.full_id)
    }
}

impl EventId {
    /// Attempts to generate an `EventId` for the given origin server with a localpart consisting
    /// of 18 random ASCII characters. This should only be used for events in the original format
    /// as used by Matrix room versions 1 and 2.
    ///
    /// Does not currently ever fail, but may fail in the future if the homeserver cannot be parsed
    /// parsed as a valid host.
    #[cfg(feature = "rand")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
    pub fn new(server_name: &ServerName) -> Self {
        use crate::generate_localpart;

        let full_id = format!("${}:{}", generate_localpart(18), server_name).into();

        Self { full_id, colon_idx: NonZeroU8::new(19) }
    }
}

impl EventId {
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

    /// Returns the server name of the event ID.
    ///
    /// Only applicable to events in the original format as used by Matrix room versions 1 and 2.
    pub fn server_name(&self) -> Option<&ServerName> {
        self.colon_idx
            .map(|idx| <&ServerName>::try_from(&self.full_id[idx.get() as usize + 1..]).unwrap())
    }
}

/// Attempts to create a new Matrix event ID from a string representation.
///
/// If using the original event format as used by Matrix room versions 1 and 2, the string must
/// include the leading $ sigil, the localpart, a literal colon, and a valid homeserver hostname.
fn try_from<S>(event_id: S) -> Result<EventId, Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let colon_idx = ruma_identifiers_validation::event_id::validate(event_id.as_ref())?;
    Ok(EventId { full_id: event_id.into(), colon_idx })
}

common_impls!(EventId, try_from, "a Matrix event ID");

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    #[cfg(feature = "serde")]
    use serde_json::{from_str, to_string};

    use super::EventId;
    use crate::Error;

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

    #[cfg(feature = "rand")]
    #[test]
    fn generate_random_valid_event_id() {
        use crate::ServerName;

        let server_name =
            <&ServerName>::try_from("example.com").expect("Failed to parse ServerName");
        let event_id = EventId::new(server_name);
        let id_str = event_id.as_str();

        assert!(id_str.starts_with('$'));
        assert_eq!(id_str.len(), 31);
    }

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "serde")]
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

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_valid_original_event_id() {
        assert_eq!(
            from_str::<EventId>(r#""$39hvsi03hlne:example.com""#)
                .expect("Failed to convert JSON to EventId"),
            EventId::try_from("$39hvsi03hlne:example.com").expect("Failed to create EventId.")
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_valid_base64_event_id() {
        assert_eq!(
            from_str::<EventId>(r#""$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk""#)
                .expect("Failed to convert JSON to EventId"),
            EventId::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                .expect("Failed to create EventId.")
        );
    }

    #[cfg(feature = "serde")]
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
            Error::MissingLeadingSigil
        );
    }

    #[test]
    fn missing_base64_event_id_sigil() {
        assert_eq!(
            EventId::try_from("acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk").unwrap_err(),
            Error::MissingLeadingSigil
        );
    }

    #[test]
    fn missing_url_safe_base64_event_id_sigil() {
        assert_eq!(
            EventId::try_from("Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg").unwrap_err(),
            Error::MissingLeadingSigil
        );
    }

    #[test]
    fn invalid_event_id_host() {
        assert_eq!(EventId::try_from("$39hvsi03hlne:/").unwrap_err(), Error::InvalidServerName);
    }

    #[test]
    fn invalid_event_id_port() {
        assert_eq!(
            EventId::try_from("$39hvsi03hlne:example.com:notaport").unwrap_err(),
            Error::InvalidServerName
        );
    }
}
