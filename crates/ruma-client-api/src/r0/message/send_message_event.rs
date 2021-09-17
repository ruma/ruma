//! [PUT /_matrix/client/r0/rooms/{roomId}/send/{eventType}/{txnId}](https://matrix.org/docs/spec/client_server/r0.6.1#put-matrix-client-r0-rooms-roomid-send-eventtype-txnid)

use ruma_api::ruma_api;
use ruma_events::{AnyMessageEventContent, MessageEventContent};
use ruma_identifiers::{EventId, RoomId};
use ruma_serde::Raw;
use serde_json::value::to_raw_value as to_raw_json_value;

ruma_api! {
    metadata: {
        description: "Send a message event to a room.",
        method: PUT,
        name: "create_message_event",
        path: "/_matrix/client/r0/rooms/:room_id/send/:event_type/:txn_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room to send the event to.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The type of event to send.
        #[ruma_api(path)]
        pub event_type: &'a str,

        /// The transaction ID for this event.
        ///
        /// Clients should generate an ID unique across requests with the
        /// same access token; it will be used by the server to ensure
        /// idempotency of requests.
        #[ruma_api(path)]
        pub txn_id: &'a str,

        /// The event content to send.
        #[ruma_api(body)]
        pub body: Raw<AnyMessageEventContent>,
    }

    response: {
        /// A unique identifier for the event.
        pub event_id: Box<EventId>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id, transaction id and event content.
    ///
    /// # Errors
    ///
    /// Since `Request` stores the request body in serialized form, this function can fail if `T`s
    /// [`Serialize`][serde::Serialize] implementation can fail.
    pub fn new<T: MessageEventContent>(
        room_id: &'a RoomId,
        txn_id: &'a str,
        content: &'a T,
    ) -> serde_json::Result<Self> {
        Ok(Self {
            room_id,
            txn_id,
            event_type: content.event_type(),
            body: Raw::from_json(to_raw_json_value(content)?),
        })
    }

    /// Creates a new `Request` with the given room id, transaction id, event type and raw event
    /// content.
    pub fn new_raw(
        room_id: &'a RoomId,
        txn_id: &'a str,
        event_type: &'a str,
        body: Raw<AnyMessageEventContent>,
    ) -> Self {
        Self { room_id, event_type, txn_id, body }
    }
}

impl Response {
    /// Creates a new `Response` with the given event id.
    pub fn new(event_id: Box<EventId>) -> Self {
        Self { event_id }
    }
}
