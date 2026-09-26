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
        serde::JsonObject,
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
        /// The room where the `m.rtc.member` event is present.
        pub room_id: OwnedRoomId,

        /// The slot ID from the `m.rtc.member` event.
        pub slot_id: String,

        /// The contents of the `member` field from the `m.rtc.member` event.
        pub member: JsonObject,

        /// The ID of the delayed `m.rtc.member` leave event to delegate.
        pub delay_id: String,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID, slot ID, member and delay ID.
        pub fn new(
            room_id: OwnedRoomId,
            slot_id: String,
            member: JsonObject,
            delay_id: String,
        ) -> Self {
            Self { room_id, slot_id, member, delay_id }
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
                owned_room_id!("!tDLCaLXijNtYcJZEey:example.com"),
                "the_id".to_owned(),
                json!({ "id": "xyzABCDEF10123" }).as_object().unwrap().clone(),
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
                    "room_id": "!tDLCaLXijNtYcJZEey:example.com",
                    "slot_id": "the_id",
                    "member": { "id": "xyzABCDEF10123" },
                    "delay_id": "1234567890",
                })
            );
        }
    }
}
