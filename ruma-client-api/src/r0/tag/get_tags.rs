//! [GET /_matrix/client/r0/user/{userId}/rooms/{roomId}/tags](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-user-userid-rooms-roomid-tags)

use ruma_api::ruma_api;
use ruma_events::tag::Tags;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata: {
        description: "Get the tags associated with a room.",
        method: GET,
        name: "get_tags",
        path: "/_matrix/client/r0/user/:user_id/rooms/:room_id/tags",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The user whose tags will be retrieved.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The room from which tags will be retrieved.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,
    }

    response: {
        /// The user's tags for the room.
        pub tags: Tags,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given user ID and room ID.
    pub fn new(user_id: &'a UserId, room_id: &'a RoomId) -> Self {
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
    use ruma_api::OutgoingResponse;
    use ruma_events::tag::{TagInfo, Tags};
    use serde_json::json;

    use super::Response;

    #[test]
    fn serializing_get_tags_response() {
        let mut tags = Tags::new();
        tags.insert("m.favourite".into(), assign!(TagInfo::new(), { order: Some(0.25) }));
        tags.insert("u.user_tag".into(), assign!(TagInfo::new(), { order: Some(0.11) }));
        let response = Response { tags };

        let http_response = response.try_into_http_response().unwrap();

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
