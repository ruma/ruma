//! Matrix event identifiers.

use ruma_macros::IdZst;

use super::ServerName;

/// A Matrix [event ID].
///
/// An `EventId` is generated randomly or converted from a string slice, and can be converted back
/// into a string as needed.
///
/// # Room versions
///
/// Matrix specifies multiple [room versions] and the format of event identifiers differ between
/// them. The original format used by room versions 1 and 2 uses a short pseudorandom "localpart"
/// followed by the hostname and port of the originating homeserver. Later room versions change
/// event identifiers to be a hash of the event encoded with Base64. Some of the methods provided by
/// `EventId` are only relevant to the original event format.
///
/// ```
/// # use ruma_common::EventId;
/// // Original format
/// assert_eq!(<&EventId>::try_from("$h29iv0s8:example.com").unwrap(), "$h29iv0s8:example.com");
/// // Room version 3 format
/// assert_eq!(
///     <&EventId>::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk").unwrap(),
///     "$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk"
/// );
/// // Room version 4 format
/// assert_eq!(
///     <&EventId>::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg").unwrap(),
///     "$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg"
/// );
/// ```
///
/// [event ID]: https://spec.matrix.org/latest/appendices/#event-ids
/// [room versions]: https://spec.matrix.org/latest/rooms/#complete-list-of-room-versions
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdZst)]
#[ruma_id(validate = ruma_identifiers_validation::event_id::validate)]
pub struct EventId(str);

impl EventId {
    /// Attempts to generate an `EventId` for the given origin server with a localpart consisting
    /// of 18 random ASCII characters.
    ///
    /// This should only be used for events in the original format  as used by Matrix room versions
    /// 1 and 2.
    #[cfg(feature = "rand")]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(server_name: &ServerName) -> OwnedEventId {
        Self::from_borrowed(&format!("${}:{server_name}", super::generate_localpart(18))).to_owned()
    }

    /// Returns the event's unique ID.
    ///
    /// For the original event format as used by Matrix room versions 1 and 2, this is the
    /// "localpart" that precedes the homeserver. For later formats, this is the entire ID without
    /// the leading `$` sigil.
    pub fn localpart(&self) -> &str {
        let idx = self.colon_idx().unwrap_or_else(|| self.as_str().len());
        &self.as_str()[1..idx]
    }

    /// Returns the server name of the event ID.
    ///
    /// Only applicable to events in the original format as used by Matrix room versions 1 and 2.
    pub fn server_name(&self) -> Option<&ServerName> {
        self.colon_idx().map(|idx| ServerName::from_borrowed(&self.as_str()[idx + 1..]))
    }

    fn colon_idx(&self) -> Option<usize> {
        self.as_str().find(':')
    }
}

#[cfg(test)]
mod tests {
    use super::{EventId, OwnedEventId};
    use crate::IdParseError;

    #[test]
    fn valid_original_event_id() {
        assert_eq!(
            <&EventId>::try_from("$39hvsi03hlne:example.com").expect("Failed to create EventId."),
            "$39hvsi03hlne:example.com"
        );
    }

    #[test]
    fn valid_base64_event_id() {
        assert_eq!(
            <&EventId>::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                .expect("Failed to create EventId."),
            "$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk"
        );
    }

    #[test]
    fn valid_url_safe_base64_event_id() {
        assert_eq!(
            <&EventId>::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                .expect("Failed to create EventId."),
            "$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg"
        );
    }

    #[cfg(feature = "rand")]
    #[test]
    fn generate_random_valid_event_id() {
        use crate::server_name;

        let event_id = EventId::new(server_name!("example.com"));
        let id_str = event_id.as_str();

        assert!(id_str.starts_with('$'));
        assert_eq!(id_str.len(), 31);
    }

    #[test]
    fn serialize_valid_original_event_id() {
        assert_eq!(
            serde_json::to_string(
                <&EventId>::try_from("$39hvsi03hlne:example.com")
                    .expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$39hvsi03hlne:example.com""#
        );
    }

    #[test]
    fn serialize_valid_base64_event_id() {
        assert_eq!(
            serde_json::to_string(
                <&EventId>::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                    .expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk""#
        );
    }

    #[test]
    fn serialize_valid_url_safe_base64_event_id() {
        assert_eq!(
            serde_json::to_string(
                <&EventId>::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                    .expect("Failed to create EventId.")
            )
            .expect("Failed to convert EventId to JSON."),
            r#""$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg""#
        );
    }

    #[test]
    fn deserialize_valid_original_event_id() {
        assert_eq!(
            serde_json::from_str::<OwnedEventId>(r#""$39hvsi03hlne:example.com""#)
                .expect("Failed to convert JSON to EventId"),
            <&EventId>::try_from("$39hvsi03hlne:example.com").expect("Failed to create EventId.")
        );
    }

    #[test]
    fn deserialize_valid_base64_event_id() {
        assert_eq!(
            serde_json::from_str::<OwnedEventId>(
                r#""$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk""#
            )
            .expect("Failed to convert JSON to EventId"),
            <&EventId>::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk")
                .expect("Failed to create EventId.")
        );
    }

    #[test]
    fn deserialize_valid_url_safe_base64_event_id() {
        assert_eq!(
            serde_json::from_str::<OwnedEventId>(
                r#""$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg""#
            )
            .expect("Failed to convert JSON to EventId"),
            <&EventId>::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg")
                .expect("Failed to create EventId.")
        );
    }

    #[test]
    fn valid_original_event_id_with_explicit_standard_port() {
        assert_eq!(
            <&EventId>::try_from("$39hvsi03hlne:example.com:443")
                .expect("Failed to create EventId."),
            "$39hvsi03hlne:example.com:443"
        );
    }

    #[test]
    fn valid_original_event_id_with_non_standard_port() {
        assert_eq!(
            <&EventId>::try_from("$39hvsi03hlne:example.com:5000")
                .expect("Failed to create EventId."),
            "$39hvsi03hlne:example.com:5000"
        );
    }

    #[test]
    fn missing_original_event_id_sigil() {
        assert_eq!(
            <&EventId>::try_from("39hvsi03hlne:example.com").unwrap_err(),
            IdParseError::MissingLeadingSigil
        );
    }

    #[test]
    fn missing_base64_event_id_sigil() {
        assert_eq!(
            <&EventId>::try_from("acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk").unwrap_err(),
            IdParseError::MissingLeadingSigil
        );
    }

    #[test]
    fn missing_url_safe_base64_event_id_sigil() {
        assert_eq!(
            <&EventId>::try_from("Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg").unwrap_err(),
            IdParseError::MissingLeadingSigil
        );
    }

    #[test]
    fn invalid_event_id_host() {
        assert_eq!(
            <&EventId>::try_from("$39hvsi03hlne:/").unwrap_err(),
            IdParseError::InvalidServerName
        );
    }

    #[test]
    fn invalid_event_id_port() {
        assert_eq!(
            <&EventId>::try_from("$39hvsi03hlne:example.com:notaport").unwrap_err(),
            IdParseError::InvalidServerName
        );
    }
}
