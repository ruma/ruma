use std::collections::BTreeMap;

use js_int::UInt;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_events::{pdu::EventHash, EventType};
use ruma_identifiers::{EventId, RoomId, ServerName, ServerSigningKeyId, UserId};
use serde_json::value::Value as JsonValue;

/// Abstraction of a PDU so users can have their own PDU types.
pub trait Event {
    /// The `EventId` of this event.
    fn event_id(&self) -> &EventId;

    /// The `RoomId` of this event.
    fn room_id(&self) -> &RoomId;

    /// The `UserId` of this event.
    fn sender(&self) -> &UserId;

    /// The time of creation on the originating server.
    fn origin_server_ts(&self) -> MilliSecondsSinceUnixEpoch;

    /// The kind of event.
    fn event_type(&self) -> &EventType;

    /// The `UserId` of this PDU.
    fn content(&self) -> serde_json::Value;

    /// The state key for this event.
    fn state_key(&self) -> Option<&str>;

    /// The events before this event.
    fn prev_events(&self) -> Vec<EventId>;

    /// The maximum number of `prev_events` plus 1.
    ///
    /// This is only used in state resolution version 1.
    fn depth(&self) -> &UInt;

    /// All the authenticating events for this event.
    fn auth_events(&self) -> Vec<EventId>;

    /// If this event is a redaction event this is the event it redacts.
    fn redacts(&self) -> Option<&EventId>;

    /// The `unsigned` content of this event.
    fn unsigned(&self) -> &BTreeMap<String, JsonValue>;

    /// The content hash of this PDU.
    fn hashes(&self) -> &EventHash;

    /// A map of server names to another map consisting of the signing key id and finally the
    /// signature.
    fn signatures(&self) -> BTreeMap<Box<ServerName>, BTreeMap<ServerSigningKeyId, String>>;
}
