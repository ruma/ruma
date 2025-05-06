//! `PUT /_matrix/client/*/rooms/{roomId}/send/{eventType}/{txnId}`
//!
//! Send a delayed event (a scheduled message) to a room. [MSC4140](https://github.com/matrix-org/matrix-spec-proposals/pull/4140)

pub mod unstable {
    //! `msc4140` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedRoomId, OwnedTransactionId,
    };
    use ruma_events::{AnyMessageLikeEventContent, MessageLikeEventContent, MessageLikeEventType};
    use serde_json::value::to_raw_value as to_raw_json_value;

    use crate::delayed_events::DelayParameters;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            // We use the unstable prefix for the delay query parameter but the stable v3 endpoint.
            unstable => "/_matrix/client/v3/rooms/:room_id/send/:event_type/:txn_id",
        }
    };
    /// Request type for the [`delayed_message_event`](crate::delayed_events::delayed_message_event)
    /// endpoint.
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

        /// The timeout duration for this delayed event.
        #[ruma_api(query_all)]
        pub delay_parameters: DelayParameters,

        /// The event content to send.
        #[ruma_api(body)]
        pub body: Raw<AnyMessageLikeEventContent>,
    }

    /// Response type for the
    /// [`delayed_message_event`](crate::delayed_events::delayed_message_event) endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The `delay_id` generated for this delayed event. Used to interact with delayed events.
        pub delay_id: String,
    }

    impl Request {
        /// Creates a new `Request` with the given room id, transaction id, `delay_parameters` and
        /// event content.
        ///
        /// # Errors
        ///
        /// Since `Request` stores the request body in serialized form, this function can fail if
        /// `T`s [`::serde::Serialize`] implementation can fail.
        pub fn new<T>(
            room_id: OwnedRoomId,
            txn_id: OwnedTransactionId,
            delay_parameters: DelayParameters,
            content: &T,
        ) -> serde_json::Result<Self>
        where
            T: MessageLikeEventContent,
        {
            Ok(Self {
                room_id,
                txn_id,
                event_type: content.event_type(),
                delay_parameters,
                body: Raw::from_json(to_raw_json_value(content)?),
            })
        }

        /// Creates a new `Request` with the given room id, transaction id, event type,
        /// `delay_parameters` and raw event content.
        pub fn new_raw(
            room_id: OwnedRoomId,
            txn_id: OwnedTransactionId,
            event_type: MessageLikeEventType,
            delay_parameters: DelayParameters,
            body: Raw<AnyMessageLikeEventContent>,
        ) -> Self {
            Self { room_id, event_type, txn_id, delay_parameters, body }
        }
    }

    impl Response {
        /// Creates a new `Response` with the tokens required to control the delayed event using the
        /// [`crate::delayed_events::update_delayed_event::unstable::Request`] request.
        pub fn new(delay_id: String) -> Self {
            Self { delay_id }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod tests {
        use ruma_common::{
            api::{MatrixVersion, OutgoingRequest, SendAccessToken, SupportedVersions},
            owned_room_id,
        };
        use ruma_events::room::message::RoomMessageEventContent;
        use serde_json::{json, Value as JsonValue};
        use web_time::Duration;

        use super::Request;
        use crate::delayed_events::delayed_message_event::unstable::DelayParameters;

        #[test]
        fn serialize_delayed_message_request() {
            let room_id = owned_room_id!("!roomid:example.org");
            let supported =
                SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Vec::new() };

            let req = Request::new(
                room_id,
                "1234".into(),
                DelayParameters::Timeout { timeout: Duration::from_millis(103) },
                &RoomMessageEventContent::text_plain("test"),
            )
            .unwrap();
            let request: http::Request<Vec<u8>> = req
                .try_into_http_request(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    &supported,
                )
                .unwrap();
            let (parts, body) = request.into_parts();
            assert_eq!(
                "https://homeserver.tld/_matrix/client/v3/rooms/!roomid:example.org/send/m.room.message/1234?org.matrix.msc4140.delay=103",
                parts.uri.to_string()
            );
            assert_eq!("PUT", parts.method.to_string());
            assert_eq!(
                json!({"msgtype":"m.text","body":"test"}),
                serde_json::from_str::<JsonValue>(std::str::from_utf8(&body).unwrap()).unwrap()
            );
        }
    }
}
