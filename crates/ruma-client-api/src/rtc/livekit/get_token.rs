//! `POST /_matrix/client/*/rtc/livekit/get_token`
//!
//! Get a token to authenticate with a LiveKit SFU for a MatrixRTC slot.

pub mod v1 {
    //! `/v1/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4195

    use ruma_common::{
        OwnedRoomId, OwnedServerName,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::JsonObject,
    };

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/io.element.msc4195/rtc/livekit/get_token",
        }
    }

    /// Request type for the `get_token` endpoint.
    #[request]
    pub struct Request {
        /// The server name of the sender of the `m.rtc.member` event.
        ///
        /// If this is `None`, the homeserver uses its own server name.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub server_name: Option<OwnedServerName>,

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
        ///
        /// The `server_name` field is set to `None`, which means that the homeserver uses its own
        /// server name.
        pub fn new(url: String, room_id: OwnedRoomId, slot_id: String, member: JsonObject) -> Self {
            Self { server_name: None, url, room_id, slot_id, member }
        }
    }

    /// Response type for the `get_token` endpoint.
    #[response]
    pub struct Response {
        /// The JWT token to use for authentication with the SFU.
        pub jwt: String,
    }

    impl Response {
        /// Creates a new `Response` with the given JWT token. The corresponding SFU url is already known via the request params.
        pub fn new(jwt: String) -> Self {
            Self { jwt }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod tests {
        use std::borrow::Cow;

        use ruma_common::{
            api::{OutgoingRequestExt as _, SupportedVersions, auth_scheme::SendAccessToken},
            owned_room_id, owned_server_name,
        };
        use serde_json::{Value as JsonValue, json};

        use super::Request;

        fn serialize_request(request: Request) -> (http::request::Parts, JsonValue) {
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

            (parts, serde_json::from_slice(&body).unwrap())
        }

        #[test]
        fn serialize_request_without_server_name() {
            let request = Request::new(
                "wss://livekit.example.com".to_owned(),
                owned_room_id!("!tDLCaLXijNtYcJZEey:example.com"),
                "the_id".to_owned(),
                json!({ "id": "xyzABCDEF10123" }).as_object().unwrap().clone(),
            );

            let (parts, body) = serialize_request(request);

            assert_eq!("POST", parts.method.as_str());
            assert_eq!(
                "https://homeserver.tld/_matrix/client/unstable/io.element.msc4195/rtc/livekit/get_token",
                parts.uri.to_string()
            );
            assert_eq!(
                body,
                json!({
                    "url": "wss://livekit.example.com",
                    "room_id": "!tDLCaLXijNtYcJZEey:example.com",
                    "slot_id": "the_id",
                    "member": { "id": "xyzABCDEF10123" },
                })
            );
        }

        #[test]
        fn serialize_request_with_server_name() {
            let mut request = Request::new(
                "wss://livekit.example.com".to_owned(),
                owned_room_id!("!tDLCaLXijNtYcJZEey:example.com"),
                "the_id".to_owned(),
                json!({ "id": "xyzABCDEF10123" }).as_object().unwrap().clone(),
            );
            request.server_name = Some(owned_server_name!("example.com"));

            let (_parts, body) = serialize_request(request);

            assert_eq!(
                body,
                json!({
                    "server_name": "example.com",
                    "url": "wss://livekit.example.com",
                    "room_id": "!tDLCaLXijNtYcJZEey:example.com",
                    "slot_id": "the_id",
                    "member": { "id": "xyzABCDEF10123" },
                })
            );
        }
    }
}
