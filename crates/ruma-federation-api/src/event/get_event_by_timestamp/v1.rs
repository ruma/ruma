//! `/v1/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1timestamp_to_eventroomid

use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, RoomId,
    api::{Direction, request, response},
    metadata,
};

use crate::authentication::ServerSignatures;

metadata! {
    method: GET,
    rate_limited: false,
    authentication: ServerSignatures,
    path: "/_matrix/federation/v1/timestamp_to_event/{room_id}",
}

/// Request type for the `get_event_by_timestamp` endpoint.
#[request]
pub struct Request {
    /// The ID of the room the event is in.
    #[ruma_api(path)]
    pub room_id: RoomId,

    /// The timestamp to search from.
    #[ruma_api(query)]
    pub ts: MilliSecondsSinceUnixEpoch,

    /// The direction in which to search.
    #[ruma_api(query)]
    pub dir: Direction,
}

/// Response type for the `get_event_by_timestamp` endpoint.
#[response]
pub struct Response {
    /// The ID of the event found.
    pub event_id: EventId,

    /// The event's timestamp.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
}

impl Request {
    /// Creates a new `Request` with the given room ID, timestamp and direction.
    pub fn new(room_id: RoomId, ts: MilliSecondsSinceUnixEpoch, dir: Direction) -> Self {
        Self { room_id, ts, dir }
    }
}

impl Response {
    /// Creates a new `Response` with the given event ID and timestamp.
    pub fn new(event_id: EventId, origin_server_ts: MilliSecondsSinceUnixEpoch) -> Self {
        Self { event_id, origin_server_ts }
    }
}
