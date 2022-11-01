//! `/v2/` ([spec])
//!
//! [spec]: https://spec.matrix.org/v1.4/server-server-api/#put_matrixfederationv2inviteroomideventid

use ruma_common::{
    api::{request, response, Metadata},
    events::AnyStrippedStateEvent,
    metadata,
    serde::Raw,
    EventId, RoomId, RoomVersionId,
};
use serde_json::value::RawValue as RawJsonValue;

const METADATA: Metadata = metadata! {
    description: "Invites a remote user to a room.",
    method: PUT,
    name: "create_invite",
    rate_limited: false,
    authentication: ServerSignatures,
    history: {
        1.0 => "/_matrix/federation/v2/invite/:room_id/:event_id",
    }
};

#[request]
pub struct Request<'a> {
    /// The room ID that the user is being invited to.
    #[ruma_api(path)]
    pub room_id: &'a RoomId,

    /// The event ID for the invite event, generated by the inviting server.
    #[ruma_api(path)]
    pub event_id: &'a EventId,

    /// The version of the room where the user is being invited to.
    pub room_version: &'a RoomVersionId,

    /// The invite event which needs to be signed.
    pub event: &'a RawJsonValue,

    /// An optional list of simplified events to help the receiver of the invite identify the room.
    pub invite_room_state: &'a [Raw<AnyStrippedStateEvent>],
}

#[response]
pub struct Response {
    /// The signed invite event.
    pub event: Box<RawJsonValue>,
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID, event ID, room version, event and invite
    /// room state.
    pub fn new(
        room_id: &'a RoomId,
        event_id: &'a EventId,
        room_version: &'a RoomVersionId,
        event: &'a RawJsonValue,
        invite_room_state: &'a [Raw<AnyStrippedStateEvent>],
    ) -> Self {
        Self { room_id, event_id, room_version, event, invite_room_state }
    }
}

impl Response {
    /// Creates a new `Response` with the given invite event.
    pub fn new(event: Box<RawJsonValue>) -> Self {
        Self { event }
    }
}
