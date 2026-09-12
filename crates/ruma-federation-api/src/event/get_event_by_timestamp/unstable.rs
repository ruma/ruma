//! `/unstable/org.matrix.msc3030/` ([MSC])
//!
//! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/3030

use ruma_common::{
    EventId, MilliSecondsSinceUnixEpoch, RoomId,
    api::{Direction, Metadata, path_builder::SinglePath, request, response},
};

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

impl Request {
    /// Creates a new `Request` with the given room ID, timestamp and direction.
    pub fn new(room_id: RoomId, ts: MilliSecondsSinceUnixEpoch, dir: Direction) -> Self {
        Self { room_id, ts, dir }
    }
}

impl Metadata for Request {
    const METHOD: http::Method = super::v1::Request::METHOD;
    const RATE_LIMITED: bool = super::v1::Request::RATE_LIMITED;
    type Authentication = <super::v1::Request as Metadata>::Authentication;
    type PathBuilder = <super::v1::Request as Metadata>::PathBuilder;
    const PATH_BUILDER: Self::PathBuilder = SinglePath::new(
        "/_matrix/federation/unstable/org.matrix.msc3030/timestamp_to_event/{room_id}",
    );
}

impl From<super::v1::Request> for Request {
    fn from(value: super::v1::Request) -> Self {
        let super::v1::Request { room_id, ts, dir } = value;
        Self { room_id, ts, dir }
    }
}

impl From<Request> for super::v1::Request {
    fn from(value: Request) -> Self {
        let Request { room_id, ts, dir } = value;
        Self { room_id, ts, dir }
    }
}

/// Response type for the `get_event_by_timestamp` endpoint.
#[response]
pub struct Response {
    /// The ID of the event found.
    pub event_id: EventId,

    /// The event's timestamp.
    pub origin_server_ts: MilliSecondsSinceUnixEpoch,
}

impl Response {
    /// Creates a new `Response` with the given event ID and timestamp.
    pub fn new(event_id: EventId, origin_server_ts: MilliSecondsSinceUnixEpoch) -> Self {
        Self { event_id, origin_server_ts }
    }
}

impl From<super::v1::Response> for Response {
    fn from(value: super::v1::Response) -> Self {
        let super::v1::Response { event_id, origin_server_ts } = value;
        Self { event_id, origin_server_ts }
    }
}

impl From<Response> for super::v1::Response {
    fn from(value: Response) -> Self {
        let Response { event_id, origin_server_ts } = value;
        Self { event_id, origin_server_ts }
    }
}
