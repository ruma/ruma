//! [GET /_matrix/federation/v1/make_knock/{roomId}/{userId}](https://github.com/matrix-org/matrix-doc/blob/master/proposals/2403-knock.md#get-_matrixfederationv1make_knockroomiduserid)

use ruma_api::ruma_api;
use ruma_events::pdu::Pdu;
use ruma_identifiers::{RoomId, RoomVersionId, UserId};
use ruma_serde::Raw;

ruma_api! {
    metadata: {
        description: "Send a request for a knock event template to a resident server.",
        name: "create_knock_event_template",
        method: GET,
        path: "/_matrix/federation/v1/make_knock/:room_id/:user_id",
        rate_limited: false,
        authentication: ServerSignatures,
    }

    request : {
        /// The room ID that should receive the knock.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The user ID the knock event will be for.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The room versions the sending has support for.
        ///
        /// Defaults to `&[RoomVersionId::Version1]
        #[ruma_api(query)]
        pub ver: &'a [RoomVersionId],
    }

    response: {
        /// The version of the room where the server is trying to knock.
        pub room_version: RoomVersionId,

        /// An unsigned template event.
        ///
        /// May differ between room versions.
        pub event: Raw<Pdu>,
    }
}
