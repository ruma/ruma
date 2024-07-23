//! `PUT /_matrix/client/*/rooms/{roomId}/state_future/{eventType}/{txnId}`
//!
//! Send a future state (a scheduled state event) to a room. [MSC](https://github.com/matrix-org/matrix-spec-proposals/pull/4140)

pub mod unstable {
    //! `msc4140` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Raw,
        OwnedRoomId,
    };
    use ruma_events::{AnyStateEventContent, StateEventContent, StateEventType};
    use serde_json::value::to_raw_value as to_raw_json_value;

    use crate::future::FutureParameters;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4140/rooms/:room_id/state_future/:event_type/:state_key",
        }
    };

    /// Request type for the [`send_future_state_event`](crate::future::send_future_state_event)
    /// endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room to send the event to.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The type of event to send.
        #[ruma_api(path)]
        pub event_type: StateEventType,

        /// The state_key for the state to send.
        #[ruma_api(path)]
        pub state_key: String,

        /// Additional parameters to describe sending a future.
        ///
        /// Only three combinations for `future_timeout` and `future_parent_id` are possible.
        /// The enum [`FutureParameters`] enforces this.
        #[ruma_api(query_all)]
        pub future_parameters: FutureParameters,

        /// The event content to send.
        #[ruma_api(body)]
        pub body: Raw<AnyStateEventContent>,
    }

    /// Response type for the [`send_future_state_event`](crate::future::send_future_state_event)
    /// endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The `future_parent_id` generated for this future. Used to connect multiple futures
        /// only one of the connected futures will be sent and inserted into the DAG.
        pub future_id: String,
    }

    impl Request {
        /// Creates a new `Request` with the given room id, state_key future_parameters and
        /// event content.
        ///
        /// # Errors
        ///
        /// Since `Request` stores the request body in serialized form, this function can fail if
        /// `T`s [`::serde::Serialize`] implementation can fail.
        pub fn new<T>(
            room_id: OwnedRoomId,
            state_key: String,
            future_parameters: FutureParameters,
            content: &T,
        ) -> serde_json::Result<Self>
        where
            T: StateEventContent,
        {
            Ok(Self {
                room_id,
                state_key,
                event_type: content.event_type(),
                future_parameters,
                body: Raw::from_json(to_raw_json_value(content)?),
            })
        }

        /// Creates a new `Request` with the given room id, event type, state key,
        /// future parameters and raw event content.
        pub fn new_raw(
            room_id: OwnedRoomId,
            state_key: String,
            event_type: StateEventType,
            future_parameters: FutureParameters,
            body: Raw<AnyStateEventContent>,
        ) -> Self {
            Self { room_id, event_type, state_key, body, future_parameters }
        }
    }

    impl Response {
        /// Creates a new `Response` with the tokens required to control the future using the
        /// [`crate::future::update_future::unstable::Request`] request.
        pub fn new(future_id: String) -> Self {
            Self { future_id }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod tests {
        use ruma_common::{
            api::{MatrixVersion, OutgoingRequest, SendAccessToken},
            owned_room_id,
        };
        use ruma_events::room::topic::RoomTopicEventContent;
        use serde_json::{json, Value as JsonValue};
        use web_time::Duration;

        use super::Request;
        use crate::future::FutureParameters;

        #[test]
        fn serialize_state_future_request() {
            let room_id = owned_room_id!("!roomid:example.org");

            let req = Request::new(
                room_id,
                "@userAsStateKey:example.org".to_owned(),
                FutureParameters::Timeout {
                    timeout: Duration::from_millis(1_234_321),
                    future_parent_id: Some("abs1abs1abs1abs1".to_owned()),
                },
                &RoomTopicEventContent::new("my_topic".to_owned()),
            )
            .unwrap();
            let request: http::Request<Vec<u8>> = req
                .try_into_http_request(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    &[MatrixVersion::V1_1],
                )
                .unwrap();
            let (parts, body) = request.into_parts();
            assert_eq!(
                "https://homeserver.tld/_matrix/client/unstable/org.matrix.msc4140/rooms/!roomid:example.org/state_future/m.room.topic/@userAsStateKey:example.org?future_timeout=1234321&future_parent_id=abs1abs1abs1abs1",
                parts.uri.to_string()
            );
            assert_eq!("PUT", parts.method.to_string());
            assert_eq!(
                json!({"topic": "my_topic"}),
                serde_json::from_str::<JsonValue>(std::str::from_utf8(&body).unwrap()).unwrap()
            );
        }
    }
}
