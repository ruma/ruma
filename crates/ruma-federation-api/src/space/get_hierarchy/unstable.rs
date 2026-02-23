//! `/unstable/org.matrix.msc2946/` ([MSC])
//!
//! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2946

use ruma_common::{
    RoomId,
    api::{Metadata, path_builder::SinglePath, request, response},
    room::RoomSummary,
};

use crate::space::SpaceHierarchyParentSummary;

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

impl Request {
    /// Creates a `Request` with the given room ID.
    pub fn new(room_id: RoomId) -> Self {
        Self { room_id, suggested_only: false }
    }
}

impl Metadata for Request {
    const METHOD: http::Method = super::v1::Request::METHOD;
    const RATE_LIMITED: bool = super::v1::Request::RATE_LIMITED;
    type Authentication = <super::v1::Request as Metadata>::Authentication;
    type PathBuilder = <super::v1::Request as Metadata>::PathBuilder;
    const PATH_BUILDER: Self::PathBuilder =
        SinglePath::new("/_matrix/federation/unstable/org.matrix.msc2946/hierarchy/{room_id}");
}

impl From<super::v1::Request> for Request {
    fn from(value: super::v1::Request) -> Self {
        let super::v1::Request { room_id, suggested_only } = value;
        Self { room_id, suggested_only }
    }
}

impl From<Request> for super::v1::Request {
    fn from(value: Request) -> Self {
        let Request { room_id, suggested_only } = value;
        Self { room_id, suggested_only }
    }
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

impl Response {
    /// Creates a new `Response` with the given room summary.
    pub fn new(room_summary: SpaceHierarchyParentSummary) -> Self {
        Self { children: Vec::new(), inaccessible_children: Vec::new(), room: room_summary }
    }
}

impl From<super::v1::Response> for Response {
    fn from(value: super::v1::Response) -> Self {
        let super::v1::Response { children, inaccessible_children, room } = value;
        Self { children, inaccessible_children, room }
    }
}

impl From<Response> for super::v1::Response {
    fn from(value: Response) -> Self {
        let Response { children, inaccessible_children, room } = value;
        Self { children, inaccessible_children, room }
    }
}
