//! `GET /_matrix/client/*/rooms/{roomId}/delayed_events`
//!
//! Get all of the user's delayed events.

pub mod unstable {
    //! `msc4140` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    use crate::delayed_events::DelayedEventData;

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/delayed_events",
        }
    }

    /// Request type for the
    /// [`get_all_delayed_events`](crate::delayed_events::get_all_delayed_events) endpoint
    #[request]
    pub struct Request {}

    /// Response type for the
    /// [`get_all_delayed_events`](crate::delayed_events::get_all_delayed_events) endpoint
    #[response]
    pub struct Response {
        /// An array of objects describing scheduled delayed events owned by the requesting user
        pub delayed_events: Vec<DelayedEventData>,
    }

    impl Request {
        /// Create a new Request
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Default for Request {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Response {
        /// Create a new Response.
        pub fn new(delayed_events: Vec<DelayedEventData>) -> Self {
            Self { delayed_events }
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod server_tests {

        use std::time::Duration;

        use js_int::UInt;
        use ruma_common::{
            MilliSecondsSinceUnixEpoch, api::OutgoingResponse, owned_event_id, owned_room_id,
            serde::Raw,
        };
        use ruma_events::TimelineEventType;
        use serde_json::{Value as JsonValue, json};

        use super::Response;
        use crate::delayed_events::DelayedEventData;

        #[test]
        fn serialize_get_all_delayed_events_response() {
            let content = json!({
                "topic": "test topic"
            })
            .to_string();

            let mut event0 = DelayedEventData::new(
                "a_delay_id".to_owned(),
                owned_room_id!("!roomid:example.org"),
                TimelineEventType::RoomTopic,
                Some("a_state_key".to_owned()),
                Raw::from_json_string(content).unwrap(),
                Duration::from_millis(103),
                MilliSecondsSinceUnixEpoch(UInt::new(70000).unwrap()),
            );

            event0.event_id = Some(owned_event_id!("$event:imaginary.hs"));
            event0.finalized_ts = Some(MilliSecondsSinceUnixEpoch(UInt::new(70103).unwrap()));

            let response = Response::new(vec![event0]);

            let response: http::Response<Vec<u8>> = response.try_into_http_response().unwrap();

            assert_eq!(
                json!({
                    "delayed_events" : [{
                        "content": {
                            "topic": "test topic"
                        },
                        "delay": 103,
                        "delay_id": "a_delay_id",
                        "event_id": "$event:imaginary.hs",
                        "finalised_ts": 70103,
                        "room_id": "!roomid:example.org",
                        "running_since": 70000,
                        "state_key": "a_state_key",
                        "type": "m.room.topic"
                        }]
                }),
                serde_json::from_slice::<JsonValue>(response.body()).unwrap()
            );
        }
    }
}
