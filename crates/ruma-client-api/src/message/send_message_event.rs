//! `PUT /_matrix/client/*/rooms/{roomId}/send/{eventType}/{txnId}`
//!
//! Send a message event to a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3roomsroomidsendeventtypetxnid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId, OwnedTransactionId,
    };
    use ruma_events::{AnyMessageLikeEventContent, MessageLikeEventContent, MessageLikeEventType};
    use serde_json::value::to_raw_value as to_raw_json_value;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/send/{event_type}/{txn_id}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/send/{event_type}/{txn_id}",
        }
    };

    /// Request type for the `create_message_event` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room to send the event to.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The type of event to send.
        #[ruma_api(path)]
        pub event_type: MessageLikeEventType,

        /// The transaction ID for this event.
        ///
        /// Clients should generate a unique ID across requests within the
        /// same session. A session is identified by an access token, and
        /// persists when the [access token is refreshed].
        ///
        /// It will be used by the server to ensure idempotency of requests.
        ///
        /// [access token is refreshed]: https://spec.matrix.org/latest/client-server-api/#refreshing-access-tokens
        #[ruma_api(path)]
        pub txn_id: OwnedTransactionId,

        /// The event content to send.
        #[ruma_api(body)]
        pub body: Raw<AnyMessageLikeEventContent>,

        /// Timestamp to use for the `origin_server_ts` of the event.
        ///
        /// This is called [timestamp massaging] and can only be used by Appservices.
        ///
        /// Note that this does not change the position of the event in the timeline.
        ///
        /// [timestamp massaging]: https://spec.matrix.org/latest/application-service-api/#timestamp-massaging
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none", rename = "ts")]
        pub timestamp: Option<MilliSecondsSinceUnixEpoch>,
    }

    /// Response type for the `create_message_event` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A unique identifier for the event.
        pub event_id: OwnedEventId,
    }

    impl Request {
        /// Creates a new `Request` with the given room id, transaction id and event content.
        ///
        /// # Errors
        ///
        /// Since `Request` stores the request body in serialized form, this function can fail if
        /// `T`s [`Serialize`][serde::Serialize] implementation can fail.
        pub fn new<T>(
            room_id: OwnedRoomId,
            txn_id: OwnedTransactionId,
            content: &T,
        ) -> serde_json::Result<Self>
        where
            T: MessageLikeEventContent,
        {
            Ok(Self {
                room_id,
                txn_id,
                event_type: content.event_type(),
                body: Raw::from_json(to_raw_json_value(content)?),
                timestamp: None,
            })
        }

        /// Creates a new `Request` with the given room id, transaction id, event type and raw event
        /// content.
        pub fn new_raw(
            room_id: OwnedRoomId,
            txn_id: OwnedTransactionId,
            event_type: MessageLikeEventType,
            body: Raw<AnyMessageLikeEventContent>,
        ) -> Self {
            Self { room_id, event_type, txn_id, body, timestamp: None }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given event id.
        pub fn new(event_id: OwnedEventId) -> Self {
            Self { event_id }
        }
    }
}
