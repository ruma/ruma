//! [PUT /_matrix/federation/v1/exchange_third_party_invite/{roomId}](https://matrix.org/docs/spec/server_server/r0.1.4#put-matrix-federation-v1-exchange-third-party-invite-roomid)

use ruma_api::ruma_api;
use ruma_events::{room::member::ThirdPartyInvite, EventType};
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata: {
        description: "The receiving server will verify the partial m.room.member event given in the request body.",
        method: PUT,
        name: "exchange_invite",
        path: "/_matrix/federation/v1/exchange_third_party_invite/:room_id",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room ID to exchange a third party invite in.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The event type. Must be `m.room.member`.
        #[serde(rename = "type")]
        pub kind: EventType,

        /// The user ID of the user who sent the original invite event.
        pub sender: &'a UserId,

        /// The user ID of the invited user.
        pub state_key: &'a UserId,

        /// The content of the invite event.
        pub content: &'a ThirdPartyInvite,
    }

    #[derive(Default)]
    response: {}
}

impl<'a> Request<'a> {
    /// Creates a new `Request` for a third party invite exchange
    pub fn new(
        room_id: &'a RoomId,
        sender: &'a UserId,
        state_key: &'a UserId,
        content: &'a ThirdPartyInvite,
    ) -> Self {
        Self { room_id, kind: EventType::RoomMember, sender, state_key, content }
    }
}

impl Response {
    /// Creates a new `Response`.
    pub fn new() -> Self {
        Self
    }
}
