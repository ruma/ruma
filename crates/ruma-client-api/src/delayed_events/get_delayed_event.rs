//! `GET /_matrix/client/*/delayed_events/{delay_id}`
//!
//! Get information about a delayed event.

pub mod msc4140 {
    //! `/unstable/org.matrix.msc4140/` ([MSC])
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
            unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/delayed_events/{delay_id}",
        }
    }

    /// Request type for the `get_delayed_event` endpoint.
    #[request]
    pub struct Request {
        /// The ID of the requested delayed event.
        #[ruma_api(path)]
        pub delay_id: String,
    }

    /// Response type for the `get_delayed_event` endpoint.
    #[response]
    pub struct Response {
        /// The returned delayed event information
        #[ruma_api(body)]
        pub delayed_event: DelayedEventData,
    }

    impl Request {
        /// Create a new Request object.
        pub fn new(delay_id: String) -> Self {
            Self { delay_id }
        }
    }

    impl Response {
        /// Create a new Response object.
        pub fn new(delayed_event: DelayedEventData) -> Self {
            Self { delayed_event }
        }
    }

    impl From<DelayedEventData> for Response {
        fn from(delayed_event: DelayedEventData) -> Self {
            Self { delayed_event }
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod server_tests {
        use std::time::Duration;

        use js_int::UInt;
        use ruma_common::{
            MilliSecondsSinceUnixEpoch, api::OutgoingResponseExt as _, owned_event_id,
            owned_room_id, serde::Raw,
        };
        use ruma_events::TimelineEventType;
        use serde_json::{Value as JsonValue, json};

        use super::Response;
        use crate::delayed_events::DelayedEventData;

        #[test]
        fn serialize_get_delayed_event_response() {
            let content = json!({
                "topic": "test topic"
            })
            .to_string();

            let mut event_data = DelayedEventData::new(
                "a_delay_id".to_owned(),
                owned_room_id!("!roomid:example.org"),
                TimelineEventType::RoomTopic,
                Some("a_state_key".to_owned()),
                Raw::from_json_string(content).unwrap(),
                Duration::from_millis(103),
                MilliSecondsSinceUnixEpoch(UInt::new(70000).unwrap()),
            );
            event_data.event_id = Some(owned_event_id!("$event:imaginary.hs"));
            event_data.finalized_ts = Some(MilliSecondsSinceUnixEpoch(UInt::new(70103).unwrap()));

            let response: http::Response<Vec<u8>> =
                Response::new(event_data).try_into_http_response().unwrap();

            assert_eq!(
                json!({
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
                }),
                serde_json::from_slice::<JsonValue>(response.body()).unwrap()
            );
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod client_tests {
        use std::time::Duration;

        use js_int::uint;
        use ruma_common::{
            MilliSecondsSinceUnixEpoch, api::IncomingResponseExt as _, owned_event_id,
        };
        use ruma_events::TimelineEventType;
        use serde_json::{Value as JsonValue, json};

        use super::Response;

        #[test]
        fn deserialize_get_delayed_event_request() {
            let body = json!({
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
            })
            .to_string();

            let res = Response::try_from_http_response(
                http::Response::builder().body(body.as_bytes()).unwrap(),
            )
            .unwrap()
            .delayed_event;

            let content = json!({
                "topic": "test topic"
            });

            assert_eq!(res.delay_id, "a_delay_id");
            assert_eq!(res.room_id, "!roomid:example.org");
            assert_eq!(res.event_type, TimelineEventType::RoomTopic);
            assert_eq!(res.state_key, Some("a_state_key".to_owned()));
            assert_eq!(res.delay, Duration::from_millis(103));
            assert_eq!(res.running_since, MilliSecondsSinceUnixEpoch(uint!(70000)));
            assert_eq!(
                serde_json::from_str::<JsonValue>(res.content.json().get()).unwrap(),
                content
            );
            assert!(res.error.is_none());
            assert_eq!(res.event_id, Some(owned_event_id!("$event:imaginary.hs")));
            assert_eq!(res.finalized_ts, Some(MilliSecondsSinceUnixEpoch(uint!(70103))));
        }
    }
}

pub mod v1 {
    //! `/v1/` ([MSC], not yet documented in spec)
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    use crate::delayed_events::DelayedEvent;

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            stable("org.matrix.msc4140.stable") => "/_matrix/client/v1/delayed_events/{delay_id}",
        }
    }

    /// Request type for the `get_delayed_event` endpoint.
    #[request]
    pub struct Request {
        /// The ID of the requested delayed event.
        #[ruma_api(path)]
        pub delay_id: String,
    }

    /// Response type for the `get_delayed_event` endpoint.
    #[response]
    pub struct Response {
        /// The requested delayed event.
        #[ruma_api(body)]
        pub delayed_event: DelayedEvent,
    }

    impl Request {
        /// Creates a new `Request` with the given delay ID.
        pub fn new(delay_id: String) -> Self {
            Self { delay_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given delayed event.
        pub fn new(delayed_event: DelayedEvent) -> Self {
            Self { delayed_event }
        }
    }

    impl From<DelayedEvent> for Response {
        fn from(delayed_event: DelayedEvent) -> Self {
            Self::new(delayed_event)
        }
    }

    impl From<super::msc4140::Request> for Request {
        fn from(request: super::msc4140::Request) -> Self {
            Self::new(request.delay_id)
        }
    }

    impl From<Request> for super::msc4140::Request {
        fn from(request: Request) -> Self {
            Self::new(request.delay_id)
        }
    }

    impl From<super::msc4140::Response> for Response {
        fn from(response: super::msc4140::Response) -> Self {
            Self::new(response.delayed_event.into())
        }
    }

    impl From<Response> for super::msc4140::Response {
        fn from(response: Response) -> Self {
            Self::new(response.delayed_event.into())
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod server_tests {
        use std::time::Duration;

        use js_int::uint;
        use ruma_common::{
            MilliSecondsSinceUnixEpoch, api::OutgoingResponseExt as _, owned_event_id,
            owned_room_id, serde::Raw,
        };
        use ruma_events::TimelineEventType;
        use serde_json::{Value as JsonValue, json};

        use super::Response;
        use crate::delayed_events::{
            DelayedEvent, DelayedEventOutcome, DelayedEventSent, FinalizedDelayedEvent,
        };

        #[test]
        fn serialize_get_delayed_event_response() {
            let mut event = DelayedEvent::new(
                "a_delay_id".to_owned(),
                owned_room_id!("!roomid:example.org"),
                TimelineEventType::RoomTopic,
                Some("".to_owned()),
                Raw::from_json_string(json!({ "topic": "test topic" }).to_string()).unwrap(),
                Duration::from_millis(103),
                MilliSecondsSinceUnixEpoch(uint!(70000)),
            );
            event.finalized = Some(FinalizedDelayedEvent::new(
                DelayedEventOutcome::Sent(DelayedEventSent::new(owned_event_id!(
                    "$event:imaginary.hs"
                ))),
                MilliSecondsSinceUnixEpoch(uint!(70103)),
            ));

            let response: http::Response<Vec<u8>> =
                Response::new(event).try_into_http_response().unwrap();

            assert_eq!(
                serde_json::from_slice::<JsonValue>(response.body()).unwrap(),
                json!({
                    "content": { "topic": "test topic" },
                    "delay_id": "a_delay_id",
                    "delay_ms": 103,
                    "delayed_since_ts": 70000,
                    "finalised": {
                        "event_id": "$event:imaginary.hs",
                        "finalised_ts": 70103,
                    },
                    "room_id": "!roomid:example.org",
                    "state_key": "",
                    "type": "m.room.topic",
                })
            );
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod client_tests {
        use std::time::Duration;

        use js_int::uint;
        use ruma_common::{MilliSecondsSinceUnixEpoch, api::IncomingResponseExt as _};
        use serde_json::json;

        use super::Response;
        use crate::delayed_events::{DelayedEventStatus, get_delayed_event::msc4140};

        #[test]
        fn deserialize_get_delayed_event_response_and_convert() {
            let body = json!({
                "content": { "topic": "test topic" },
                "delay_id": "a_delay_id",
                "delay_ms": 103,
                "delayed_since_ts": 70000,
                "finalised": {
                    "error": { "errcode": "M_FORBIDDEN", "error": "Nope" },
                    "finalised_ts": 70103,
                },
                "room_id": "!roomid:example.org",
                "type": "m.room.topic",
            })
            .to_string();

            let event = Response::try_from_http_response(
                http::Response::builder().body(body.as_bytes()).unwrap(),
            )
            .unwrap()
            .delayed_event;

            assert_eq!(event.delay_ms, Duration::from_millis(103));
            assert_eq!(event.delayed_since_ts, MilliSecondsSinceUnixEpoch(uint!(70000)));
            assert_eq!(event.status(), DelayedEventStatus::Error);

            let old = msc4140::Response::from(Response::new(event)).delayed_event;
            assert_eq!(old.delay, Duration::from_millis(103));
            assert_eq!(old.finalized_ts, Some(MilliSecondsSinceUnixEpoch(uint!(70103))));
            assert_eq!(old.status(), DelayedEventStatus::Error);

            let roundtrip = Response::from(msc4140::Response::new(old)).delayed_event;
            assert_eq!(roundtrip.status(), DelayedEventStatus::Error);
        }
    }
}
