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
    #[cfg(feature = "unstable-msc4354")]
    use ruma_events::sticky::StickyDurationMs;
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

        /// The duration to stick the event for.
        ///
        /// Valid values are the integer range 0-3600000 (1 hour)
        /// The presence of this field indicates that the event should be sticky, this
        /// will give this event additional delivery guarantees.
        ///
        /// Caller must first check that the server supports sticky events (via `/versions`),
        /// or it will be no-op.
        ///
        /// See [MSC4354 sticky events](https://github.com/matrix-org/matrix-spec-proposals/pull/4354)
        #[cfg(feature = "unstable-msc4354")]
        #[ruma_api(query)]
        #[serde(
            skip_serializing_if = "Option::is_none",
            rename = "org.matrix.msc4354.sticky_duration_ms"
        )]
        pub sticky_duration_ms: Option<StickyDurationMs>,
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
                #[cfg(feature = "unstable-msc4354")]
                sticky_duration_ms: None,
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
            Self {
                room_id,
                event_type,
                txn_id,
                body,
                timestamp: None,
                #[cfg(feature = "unstable-msc4354")]
                sticky_duration_ms: None,
            }
        }

        /// Creates a new `Request` for a sticky event with the given room id, transaction id, event
        /// type and raw event content.
        #[cfg(feature = "unstable-msc4354")]
        pub fn new_raw_sticky(
            room_id: OwnedRoomId,
            txn_id: OwnedTransactionId,
            event_type: MessageLikeEventType,
            body: Raw<AnyMessageLikeEventContent>,
            sticky_duration_ms: StickyDurationMs,
        ) -> Self {
            Self {
                room_id,
                event_type,
                txn_id,
                body,
                timestamp: None,
                sticky_duration_ms: Some(sticky_duration_ms),
            }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given event id.
        pub fn new(event_id: OwnedEventId) -> Self {
            Self { event_id }
        }
    }

    #[cfg(test)]
    mod tests {

        #[test]
        #[cfg(feature = "unstable-msc4354")]
        fn test_sticky_request() {
            use ruma_common::{
                api::{MatrixVersion, OutgoingRequest, SendAccessToken, SupportedVersions},
                owned_room_id,
            };
            use serde_json::json;

            use super::*;

            let supported = SupportedVersions {
                versions: [MatrixVersion::V1_1].into(),
                features: Default::default(),
            };

            let http_request: http::Request<Vec<u8>> = Request::new_raw_sticky(
                owned_room_id!("!roomid:example.org"),
                "0000".into(),
                MessageLikeEventType::RoomMessage,
                Raw::new(&json!({ "body": "Hello" })).unwrap().cast_unchecked(),
                StickyDurationMs::new_wrapping(123_456_u32),
            )
            .try_into_http_request(
                "https://homeserver.tld",
                SendAccessToken::IfRequired("auth_tok"),
                &supported,
            )
            .unwrap();

            assert_eq!(
                http_request.uri().query().unwrap(),
                "org.matrix.msc4354.sticky_duration_ms=123456"
            );
        }
    }
}
