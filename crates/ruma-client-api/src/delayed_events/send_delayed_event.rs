//! `PUT /_matrix/client/*/rooms/{roomId}/delayed_event/{eventType}/{txnId}`
//!
//! Send a delayed event (a scheduled message) to a room.

pub mod unstable {
    //! `msc4140` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use std::time::Duration;

    use ruma_common::{
        OwnedRoomId, OwnedTransactionId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::Raw,
    };
    #[cfg(feature = "unstable-msc4354")]
    use ruma_events::sticky::StickyDurationMs;
    use ruma_events::{AnyTimelineEventContent, TimelineEventType};

    metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/rooms/{room_id}/delayed_event/{event_type}/{txn_id}",
        }
    }
    /// Request type for the [`send_delayed_event`](crate::delayed_events::send_delayed_event)
    /// endpoint.
    #[request]
    pub struct Request {
        /// The room to send the event to.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The type of event to send.
        #[ruma_api(path)]
        pub event_type: TimelineEventType,

        /// The transaction ID for this event.
        ///
        /// Clients should generate a unique ID across requests within the
        /// same session. A session is identified by an access token, and
        /// persists when the [access token is refreshed].
        ///
        /// It will be used by the server to ensure idempotency of requests.
        ///
        /// [access token is refreshed]: https://spec.matrix.org/v1.19/client-server-api/#refreshing-access-tokens
        #[ruma_api(path)]
        pub txn_id: OwnedTransactionId,

        /// The duration that the server should wait before sending this event
        #[serde(with = "ruma_common::serde::duration::ms")]
        pub delay: Duration,

        /// The duration to stick the delayed event.
        ///
        /// Caller must first check that the server supports sticky events (via `/versions`),
        /// or it will be no-op.
        ///
        /// See [MSC4354 sticky events](https://github.com/matrix-org/matrix-spec-proposals/pull/4354).
        #[cfg(feature = "unstable-msc4354")]
        #[ruma_api(query)]
        #[serde(
            skip_serializing_if = "Option::is_none",
            rename = "org.matrix.msc4354.sticky_duration_ms"
        )]
        pub sticky_duration_ms: Option<StickyDurationMs>,

        /// The State Key if the event is a state event, nothing otherwise
        #[serde(skip_serializing_if = "Option::is_none")]
        pub state_key: Option<String>,

        /// The event content to send.
        pub content: Raw<AnyTimelineEventContent>,
    }

    /// Response type for the
    /// [`send_delayed_event`](crate::delayed_events::send_delayed_event) endpoint.
    #[response]
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
        pub fn new(
            room_id: OwnedRoomId,
            txn_id: OwnedTransactionId,
            delay: Duration,
            state_key: Option<String>,
            content: &AnyTimelineEventContent,
        ) -> serde_json::Result<Self> {
            Ok(Self {
                room_id,
                txn_id,
                event_type: content.event_type(),
                state_key,
                delay,
                #[cfg(feature = "unstable-msc4354")]
                sticky_duration_ms: None,
                content: Raw::new(content)?,
            })
        }

        /// Creates a new `Request` with the given room id, transaction id, event type,
        /// `delay_parameters` and raw event content.
        pub fn new_raw(
            event_type: TimelineEventType,
            room_id: OwnedRoomId,
            txn_id: OwnedTransactionId,
            delay: Duration,
            state_key: Option<String>,
            content: Raw<AnyTimelineEventContent>,
        ) -> serde_json::Result<Self> {
            Ok(Self {
                room_id,
                txn_id,
                event_type,
                state_key,
                delay,
                #[cfg(feature = "unstable-msc4354")]
                sticky_duration_ms: None,
                content,
            })
        }
    }

    impl Response {
        /// Creates a new `Response` with the tokens required to control the delayed event using the
        /// [`crate::delayed_events::update_delayed_event::unstable_v2::Request`] request.
        pub fn new(delay_id: String) -> Self {
            Self { delay_id }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod client_tests {
        use std::borrow::Cow;

        use ruma_common::{
            api::{
                MatrixVersion, OutgoingRequestExt as _, SupportedVersions,
                auth_scheme::SendAccessToken,
            },
            owned_room_id,
        };
        use ruma_events::{AnyMessageLikeEventContent, room::message::RoomMessageEventContent};
        use serde_json::{Value as JsonValue, json};
        use web_time::Duration;

        use super::Request;

        #[test]
        fn serialize_send_delayed_event_request() {
            let room_id = owned_room_id!("!roomid:example.org");
            let supported = SupportedVersions {
                versions: [MatrixVersion::V1_1].into(),
                features: Default::default(),
            };

            let req = Request::new(
                room_id,
                "1234".into(),
                Duration::from_millis(103),
                None,
                &AnyMessageLikeEventContent::from(RoomMessageEventContent::text_plain("test"))
                    .into(),
            )
            .unwrap();
            let request: http::Request<Vec<u8>> = req
                .try_into_http_request(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    Cow::Owned(supported),
                )
                .unwrap();
            let (parts, body) = request.into_parts();
            assert_eq!(
                "https://homeserver.tld/_matrix/client/unstable/org.matrix.msc4140/rooms/!roomid:example.org/delayed_event/m.room.message/1234",
                parts.uri.to_string()
            );
            assert_eq!("PUT", parts.method.to_string());
            assert_eq!(
                json!({"content":{"msgtype":"m.text","body":"test"}, "delay": 103}),
                serde_json::from_str::<JsonValue>(std::str::from_utf8(&body).unwrap()).unwrap()
            );
        }

        #[cfg(feature = "unstable-msc4354")]
        #[test]
        fn serialize_send_delayed_sticky_event_request() {
            use ruma_events::sticky::StickyDurationMs;

            let supported = SupportedVersions {
                versions: [MatrixVersion::V1_1].into(),
                features: Default::default(),
            };

            let mut req = Request::new(
                owned_room_id!("!roomid:example.org"),
                "1234".into(),
                Duration::from_millis(30_000),
                None,
                &AnyMessageLikeEventContent::from(RoomMessageEventContent::text_plain("test"))
                    .into(),
            )
            .unwrap();
            req.sticky_duration_ms = Some(StickyDurationMs::new_clamped(300_000_u32));

            let request: http::Request<Vec<u8>> = req
                .try_into_http_request(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    Cow::Owned(supported),
                )
                .unwrap();
            let (parts, body) = request.into_parts();
            assert_eq!(
                "https://homeserver.tld/_matrix/client/unstable/org.matrix.msc4140/rooms/!roomid:example.org/delayed_event/m.room.message/1234?org.matrix.msc4354.sticky_duration_ms=300000",
                parts.uri.to_string()
            );
            assert_eq!(
                json!({"content":{"msgtype":"m.text","body":"test"}, "delay": 30000}),
                serde_json::from_str::<JsonValue>(std::str::from_utf8(&body).unwrap()).unwrap()
            );
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod server_tests {

        use std::time::Duration;

        use ruma_common::{OwnedTransactionId, api::IncomingRequestExt as _, owned_room_id};
        use serde_json::json;

        use super::Request;

        #[test]
        fn deserialize_send_delayed_events_request() {
            let uri = http::Uri::builder()
                .scheme("https")
                .authority("matrix.org")
                .path_and_query(
                    "/_matrix/client/unstable/org.matrix.msc4140/rooms/!roomid:example.org/delayed_event/m.room.message/5678",
                )
                .build()
                .unwrap();

            let body = json!({"content":{"msgtype":"m.text","body":"test"}, "delay": 103});

            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method("PUT")
                    .uri(uri)
                    .body(body.to_string().as_bytes())
                    .unwrap(),
                &["!roomid:example.org", "m.room.message", "5678"],
            )
            .unwrap();

            assert_eq!(req.room_id, owned_room_id!("!roomid:example.org"));
            assert_eq!(req.event_type, "m.room.message".into());
            assert_eq!(req.txn_id, OwnedTransactionId::from("5678"));
            assert_eq!(req.delay, Duration::from_millis(103));
            assert_eq!(req.state_key, None);
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(req.content.json().get()).unwrap(),
                json!({"msgtype":"m.text","body":"test"}),
            );
        }

        /// Without the query parameter the delayed event is scheduled as a regular, non-sticky
        /// event.
        #[cfg(feature = "unstable-msc4354")]
        #[test]
        fn deserialize_send_delayed_event_request_without_sticky() {
            let uri = http::Uri::builder()
                .scheme("https")
                .authority("matrix.org")
                .path_and_query(
                    "/_matrix/client/unstable/org.matrix.msc4140/rooms/!roomid:example.org/delayed_event/m.room.message/5678",
                )
                .build()
                .unwrap();

            let body = json!({"content":{"msgtype":"m.text","body":"test"}, "delay": 103});

            let req = Request::try_from_http_request(
                http::Request::builder()
                    .method("PUT")
                    .uri(uri)
                    .body(body.to_string().as_bytes())
                    .unwrap(),
                &["!roomid:example.org", "m.room.message", "5678"],
            )
            .unwrap();

            assert_eq!(req.sticky_duration_ms, None);
        }
    }
}
