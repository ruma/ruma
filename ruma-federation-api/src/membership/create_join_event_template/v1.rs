//! [GET /_matrix/federation/v1/make_join/{roomId}/{userId}](https://matrix.org/docs/spec/server_server/r0.1.3#get-matrix-federation-v1-make-join-roomid-userid)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_events::EventJson;
use ruma_identifiers::{RoomId, UserId};

use crate::pdu::Pdu;

ruma_api! {
    metadata {
        description: "Send a request for a join event template to a resident server.",
        name: "create_join_event_template",
        method: GET,
        path: "/_matrix/federation/v1/make_join/:room_id/:user_id",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room ID that is about to be joined.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The user ID the join event will be for.
        #[ruma_api(path)]
        pub user_id: UserId,
        #[ruma_api(query)]
        /// The room versions the sending server has support for. Defaults to [1].
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub ver: Vec<UInt>,
    }

    response {
        /// The version of the room where the server is trying to join.
        pub room_version: Option<UInt>,
        /// An unsigned template event.
        pub event: EventJson<Pdu>,
    }
}
