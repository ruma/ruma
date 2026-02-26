//! Matrix event identifiers.

use ruma_macros::IdDst;

use super::{IdParseError, ServerName};

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
/// # use ruma_common::{server_name, EventId};
/// // Room versions 1 and 2
/// assert_eq!(<&EventId>::try_from("$h29iv0s8:example.com").unwrap(), "$h29iv0s8:example.com");
///
/// # #[cfg(feature = "rand")]
/// # {
/// let server_name = server_name!("example.com");
/// let event_id = EventId::new_v1(server_name);
/// assert_eq!(event_id.localpart().len(), 18);
/// assert_eq!(event_id.server_name(), Some(server_name));
/// # }
///
/// // Room version 3
/// assert_eq!(
///     <&EventId>::try_from("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk").unwrap(),
///     "$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk"
/// );
/// assert_eq!(
///     EventId::new_v2_or_v3("acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk").unwrap(),
///     "$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk"
/// );
///
/// // Room version 4 and later
/// assert_eq!(
///     <&EventId>::try_from("$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg").unwrap(),
///     "$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg"
/// );
/// assert_eq!(
///     EventId::new_v2_or_v3("Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg").unwrap(),
///     "$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg"
/// );
/// ```
///
/// [event ID]: https://spec.matrix.org/latest/appendices/#event-ids
/// [room versions]: https://spec.matrix.org/latest/rooms/#complete-list-of-room-versions
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, IdDst)]
#[ruma_id(validate = ruma_identifiers_validation::event_id::validate)]
pub struct EventId(str);

impl EventId {
    /// Attempts to generate an `OwnedEventId` for the given origin server with a localpart
    /// consisting of 18 random ASCII characters.
    ///
    /// This generates an event ID matching the [`EventIdFormatVersion::V1`] variant of the
    /// `event_id_format` field of [`RoomVersionRules`]. To construct an event ID matching the
    /// [`EventIdFormatVersion::V2`] or [`EventIdFormatVersion::V3`] variants, use
    /// [`EventId::new_v2_or_v3()`] instead.
    ///
    /// [`EventIdFormatVersion::V1`]: crate::room_version_rules::EventIdFormatVersion::V1
    /// [`EventIdFormatVersion::V2`]: crate::room_version_rules::EventIdFormatVersion::V2
    /// [`EventIdFormatVersion::V3`]: crate::room_version_rules::EventIdFormatVersion::V3
    /// [`RoomVersionRules`]: crate::room_version_rules::RoomVersionRules
    #[cfg(feature = "rand")]
    #[allow(clippy::new_ret_no_self)]
    pub fn new_v1(server_name: &ServerName) -> OwnedEventId {
        OwnedEventId::from_string_unchecked(format!(
            "${}:{server_name}",
            super::generate_localpart(18)
        ))
    }

    /// Construct an `OwnedEventId` using the reference hash of the event.
    ///
    /// This generates a room ID matching the [`EventIdFormatVersion::V2`] or
    /// [`EventIdFormatVersion::V3`] variants of the `event_id_format` field of
    /// [`RoomVersionRules`]. To construct an event ID matching the [`EventIdFormatVersion::V1`]
    /// variant, use [`EventId::new_v1()`] instead.
    ///
    /// Returns an error if the given string contains a NUL byte or is too long.
    ///
    /// [`EventIdFormatVersion::V1`]: crate::room_version_rules::EventIdFormatVersion::V1
    /// [`EventIdFormatVersion::V2`]: crate::room_version_rules::EventIdFormatVersion::V2
    /// [`EventIdFormatVersion::V3`]: crate::room_version_rules::EventIdFormatVersion::V3
    /// [`RoomVersionRules`]: crate::room_version_rules::RoomVersionRules
    pub fn new_v2_or_v3(reference_hash: &str) -> Result<OwnedEventId, IdParseError> {
        OwnedEventId::try_from(format!("${reference_hash}"))
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
        self.colon_idx().map(|idx| ServerName::from_borrowed_unchecked(&self.as_str()[idx + 1..]))
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

        let server_name = server_name!("example.com");
        let event_id = EventId::new_v1(server_name);
        let id_str = event_id.as_str();

        assert!(id_str.starts_with('$'));
        assert_eq!(id_str.len(), 31);
        assert_eq!(event_id.server_name(), Some(server_name));
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

    #[test]
    fn construct_v2_or_v3_event_id() {
        assert_eq!(
            EventId::new_v2_or_v3("Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg").unwrap(),
            "$Rqnc-F-dvnEYJTyHq_iKxU2bZ1CI92-kuZq3a5lr5Zg"
        );
    }
}
