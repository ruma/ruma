//! `/v1/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1hierarchyroomid

use ruma_common::{
    RoomId,
    api::{request, response},
    metadata,
    room::RoomSummary,
};

use crate::{authentication::ServerSignatures, space::SpaceHierarchyParentSummary};

metadata! {
    method: GET,
    rate_limited: false,
    authentication: ServerSignatures,
    path: "/_matrix/federation/v1/hierarchy/{room_id}",
}

/// Request type for the `hierarchy` endpoint.
#[request]
pub struct Request {
    /// The room ID of the space to get a hierarchy for.
    #[ruma_api(path)]
    pub room_id: RoomId,

    /// Whether or not the server should only consider suggested rooms.
    ///
    /// Suggested rooms are annotated in their `m.space.child` event contents.
    #[ruma_api(query)]
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub suggested_only: bool,
}

/// Response type for the `hierarchy` endpoint.
#[response]
pub struct Response {
    /// A summary of the space’s children.
    ///
    /// Rooms which the requesting server cannot peek/join will be excluded.
    pub children: Vec<RoomSummary>,

    /// The list of room IDs the requesting server doesn’t have a viable way to peek/join.
    ///
    /// Rooms which the responding server cannot provide details on will be outright
    /// excluded from the response instead.
    pub inaccessible_children: Vec<RoomId>,

    /// A summary of the requested room.
    pub room: SpaceHierarchyParentSummary,
}

impl Request {
    /// Creates a `Request` with the given room ID.
    pub fn new(room_id: RoomId) -> Self {
        Self { room_id, suggested_only: false }
    }
}

impl Response {
    /// Creates a new `Response` with the given room summary.
    pub fn new(room_summary: SpaceHierarchyParentSummary) -> Self {
        Self { children: Vec::new(), inaccessible_children: Vec::new(), room: room_summary }
    }
}

#[cfg(all(test, feature = "client"))]
mod tests {
    use ruma_common::{RoomId, api::IncomingResponse};
    use serde_json::{json, to_vec as to_json_vec};

    use super::Response;

    #[test]
    fn deserialize_response() {
        let body = json!({
            "children": [
                {
                    "room_id": "!a:localhost",
                    "num_joined_members": 6,
                    "world_readable": true,
                    "guest_can_join": false,
                    "join_rule": "public",
                },
            ],
            "inaccessible_children": [],
            "room": {
                "room_id": "!room:localhost",
                "num_joined_members": 5,
                "world_readable": false,
                "guest_can_join": false,
                "join_rule": "restricted",
                "allowed_room_ids": ["!otherroom:localhost"],
                "type": "space",
                "children_state": [
                    {
                        "content": {
                            "via": [
                                "example.org"
                            ]
                        },
                        "origin_server_ts": 1_629_413_349,
                        "sender": "@alice:example.org",
                        "state_key": "!a:example.org",
                        "type": "m.space.child"
                    }
                ],
            },
        });
        let response = http::Response::new(to_json_vec(&body).unwrap());

        let response = Response::try_from_http_response(response).unwrap();

        assert_eq!(response.room.summary.room_id, "!room:localhost");
        let space_child = response.room.children_state[0].deserialize().unwrap();
        assert_eq!(space_child.state_key, "!a:example.org");
        assert_eq!(response.inaccessible_children, &[] as &[RoomId]);
        assert_eq!(response.children[0].room_id, "!a:localhost");
    }
}
