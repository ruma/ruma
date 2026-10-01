//! `POST /_matrix/client/*/rtc/livekit/delegate_delayed_leave`
//!
//! Delegate the restarting of the delayed leave event of a MatrixRTC slot to the homeserver.

pub mod v1 {
    //! `/v1/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4195

    use ruma_common::{
        OwnedRoomId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/io.element.msc4195/rtc/livekit/delegate_delayed_leave",
        }
    }

    /// Request type for the `delegate_delayed_leave` endpoint.
    #[request]
    pub struct Request {
        /// The WebSocket URL of the LiveKit SFU that the user has connected to.
        pub url: String,

        /// The room where the `m.rtc.member` event is present.
        pub room_id: OwnedRoomId,

        /// The slot ID from the `m.rtc.member` event.
        pub slot_id: String,

        /// The `member.id` property of the `m.rtc.member` event.
        pub member_id: String,

        /// The ID of the delayed `m.rtc.member` leave event to delegate.
        pub delay_id: String,
    }

    impl Request {
        /// Creates a new `Request` with the given SFU URL, room ID, slot ID, member ID and delay
        /// ID.
        pub fn new(
            url: String,
            room_id: OwnedRoomId,
            slot_id: String,
            member_id: String,
            delay_id: String,
        ) -> Self {
            Self { url, room_id, slot_id, member_id, delay_id }
        }
    }

    /// Response type for the `delegate_delayed_leave` endpoint.
    #[response]
    pub struct Response {}

    impl Response {
        /// Creates a new empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod tests {
        use std::borrow::Cow;

        use ruma_common::{
            api::{OutgoingRequestExt as _, SupportedVersions, auth_scheme::SendAccessToken},
            owned_room_id,
        };
        use serde_json::json;

        use super::Request;

        #[test]
        fn serialize_request() {
            let request = Request::new(
                "wss://livekit.example.com".to_owned(),
                owned_room_id!("!tDLCaLXijNtYcJZEey:example.com"),
                "the_id".to_owned(),
                "xyzABCDEF10123".to_owned(),
                "1234567890".to_owned(),
            );

            let supported =
                SupportedVersions { versions: Default::default(), features: Default::default() };

            let (parts, body) = request
                .try_into_http_request::<Vec<u8>>(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    Cow::Owned(supported),
                )
                .unwrap()
                .into_parts();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert_eq!("POST", parts.method.as_str());
            assert_eq!(
                "https://homeserver.tld/_matrix/client/unstable/io.element.msc4195/rtc/livekit/delegate_delayed_leave",
                parts.uri.to_string()
            );
            assert_eq!(
                body,
                json!({
                    "url": "wss://livekit.example.com",
                    "room_id": "!tDLCaLXijNtYcJZEey:example.com",
                    "slot_id": "the_id",
                    "member_id": "xyzABCDEF10123",
                    "delay_id": "1234567890",
                })
            );
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod server_tests {
        use ruma_common::api::IncomingRequestExt as _;

        use super::Request;

        #[test]
        fn deserialize_request() {
            let request = http::Request::builder()
                .method("POST")
                .uri(
                    "/_matrix/client/unstable/io.element.msc4195/rtc/livekit/delegate_delayed_leave",
                )
                .body(
                    br#"{
                        "url": "wss://livekit.example.org",
                        "room_id": "!tDLCaLXijNtYcJZEey:example.org",
                        "slot_id": "the_id",
                        "member_id": "xyzABCDEF10123",
                        "delay_id": "1234567890"
                    }"# as &[u8],
                )
                .unwrap();

            let request = Request::try_from_http_request(request, &[] as &[&str]).unwrap();

            assert_eq!(request.url, "wss://livekit.example.org");
            assert_eq!(request.room_id, "!tDLCaLXijNtYcJZEey:example.org");
            assert_eq!(request.slot_id, "the_id");
            assert_eq!(request.member_id, "xyzABCDEF10123");
            assert_eq!(request.delay_id, "1234567890");
        }
    }
}
