//! `GET /_matrix/client/*/user/{userId}/rooms/{roomId}/tags`
//!
//! Get the tags associated with a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3useruseridroomsroomidtags

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedUserId,
    };
    use ruma_events::tag::Tags;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/user/{user_id}/rooms/{room_id}/tags",
            1.1 => "/_matrix/client/v3/user/{user_id}/rooms/{room_id}/tags",
        }
    };

    /// Request type for the `get_tags` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose tags will be retrieved.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The room from which tags will be retrieved.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,
    }

    /// Response type for the `get_tags` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The user's tags for the room.
        pub tags: Tags,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID and room ID.
        pub fn new(user_id: OwnedUserId, room_id: OwnedRoomId) -> Self {
            Self { user_id, room_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given tags.
        pub fn new(tags: Tags) -> Self {
            Self { tags }
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod server_tests {
        use assign::assign;
        use ruma_common::api::OutgoingResponse;
        use ruma_events::tag::{TagInfo, Tags};
        use serde_json::json;

        use super::Response;

        #[test]
        fn serializing_get_tags_response() {
            let mut tags = Tags::new();
            tags.insert("m.favourite".into(), assign!(TagInfo::new(), { order: Some(0.25) }));
            tags.insert("u.user_tag".into(), assign!(TagInfo::new(), { order: Some(0.11) }));
            let response = Response { tags };

            let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

            let json_response: serde_json::Value =
                serde_json::from_slice(http_response.body()).unwrap();
            assert_eq!(
                json_response,
                json!({
                    "tags": {
                        "m.favourite": {
                            "order": 0.25,
                        },
                        "u.user_tag": {
                            "order": 0.11,
                        }
                    }
                })
            );
        }
    }
}
