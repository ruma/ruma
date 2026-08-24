//! `POST /_matrix/federation/*/rtc/livekit/get_token`
//!
//! Get a token to authenticate with a LiveKit SFU of the receiving server for a MatrixRTC slot.

pub mod msc4195 {
    //! `MSC4195` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4195

    use ruma_common::{
        OwnedRoomId,
        api::{request, response},
        metadata,
        serde::JsonObject,
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
        /// The WebSocket URL of the LiveKit SFU.
        pub url: String,

        /// The room where the `m.rtc.member` event is present.
        pub room_id: OwnedRoomId,

        /// The slot ID from the `m.rtc.member` event.
        pub slot_id: String,

        /// The contents of the `member` field from the `m.rtc.member` event.
        pub member: JsonObject,
    }

    impl Request {
        /// Creates a new `Request` with the given SFU URL, room ID, slot ID and member.
        pub fn new(url: String, room_id: OwnedRoomId, slot_id: String, member: JsonObject) -> Self {
            Self { url, room_id, slot_id, member }
        }
    }

    /// Response type for the `get_token` endpoint.
    #[response]
    pub struct Response {
        /// The JWT token to use for authentication with the SFU.
        pub jwt: String,
    }

    impl Response {
        /// Creates a new `Response` with the given JWT and SFU URL.
        pub fn new(jwt: String) -> Self {
            Self { jwt }
        }
    }
}
