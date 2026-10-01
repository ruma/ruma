//! `POST /_matrix/federation/*/rtc/livekit/get_token`
//!
//! Get a token to authenticate with a LiveKit SFU of the receiving server for a MatrixRTC slot.

pub mod msc4195 {
    //! `MSC4195` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4195

    use ruma_common::{
        OwnedRoomId, OwnedUserId,
        api::{request, response},
        metadata,
    };

    use crate::authentication::ServerSignatures;

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/unstable/io.element.msc4195/rtc/livekit/get_token",
    }

    /// Request type for the `get_token` endpoint.
    #[request]
    pub struct Request {
        /// The Matrix ID of the user requesting the token.
        ///
        /// Must belong to the origin server.
        pub user_id: OwnedUserId,

        /// The WebSocket URL of the LiveKit SFU.
        pub url: String,

        /// The room where the `m.rtc.member` event is present.
        pub room_id: OwnedRoomId,

        /// The slot ID from the `m.rtc.member` event.
        pub slot_id: String,

        /// The `member.id` property of the requesting user's own `m.rtc.member` event.
        pub member_id: String,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID, SFU URL, room ID, slot ID and member
        /// ID.
        pub fn new(
            user_id: OwnedUserId,
            url: String,
            room_id: OwnedRoomId,
            slot_id: String,
            member_id: String,
        ) -> Self {
            Self { user_id, url, room_id, slot_id, member_id }
        }
    }

    /// Response type for the `get_token` endpoint.
    #[response]
    pub struct Response {
        /// The JWT token to use for authentication with the SFU.
        pub jwt: String,
    }

    impl Response {
        /// Creates a new `Response` with the given JWT token. The corresponding SFU url is already
        /// known via the request params.
        pub fn new(jwt: String) -> Self {
            Self { jwt }
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
                .uri("/_matrix/federation/unstable/io.element.msc4195/rtc/livekit/get_token")
                .body(
                    br#"{
                        "user_id": "@alice:example.org",
                        "url": "wss://livekit.example.org",
                        "room_id": "!tDLCaLXijNtYcJZEey:example.org",
                        "slot_id": "the_id",
                        "member_id": "xyzABCDEF10123"
                    }"# as &[u8],
                )
                .unwrap();

            let request = Request::try_from_http_request(request, &[] as &[&str]).unwrap();

            assert_eq!(request.user_id, "@alice:example.org");
            assert_eq!(request.url, "wss://livekit.example.org");
            assert_eq!(request.room_id, "!tDLCaLXijNtYcJZEey:example.org");
            assert_eq!(request.slot_id, "the_id");
            assert_eq!(request.member_id, "xyzABCDEF10123");
        }
    }
}
