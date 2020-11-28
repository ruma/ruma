//! [GET /_matrix/federation/v1/make_join/{roomId}/{userId}](https://matrix.org/docs/spec/server_server/r0.1.3#get-matrix-federation-v1-make-join-roomid-userid)

use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{RoomId, RoomVersionId, UserId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Send a request for a join event template to a resident server.",
        name: "create_join_event_template",
        method: GET,
        path: "/_matrix/federation/v1/make_join/:room_id/:user_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request: {
        /// The room ID that is about to be joined.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The user ID the join event will be for.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The room versions the sending server has support for.
        ///
        /// Defaults to `&[RoomVersionId::Version1]`.
        #[ruma_api(query)]
        #[serde(default = "default_ver", skip_serializing_if = "is_default_ver")]
        pub ver: &'a [RoomVersionId],
    }

    response: {
        /// The version of the room where the server is trying to join.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_version: Option<RoomVersionId>,

        /// An unsigned template event.
        pub event: Raw<Pdu>,
    }
}

fn default_ver() -> Vec<RoomVersionId> {
    vec![RoomVersionId::Version1]
}

fn is_default_ver(ver: &&[RoomVersionId]) -> bool {
    **ver == [RoomVersionId::Version1]
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room id and user id.
    pub fn new(room_id: &'a RoomId, user_id: &'a UserId) -> Self {
        Self { room_id, user_id, ver: &[RoomVersionId::Version1] }
    }
}

impl Response {
    /// Creates a new `Response` with the given template event.
    pub fn new(event: Raw<Pdu>) -> Self {
        Self { room_version: None, event }
    }
}
