//! `/unstable/xyz.amorgan.knock/` ([MSC])
//!
//! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2403

use ruma_common::{
    RoomId, RoomVersionId, UserId,
    api::{Metadata, path_builder::SinglePath, request, response},
};
use serde_json::value::RawValue as RawJsonValue;

/// Request type for the `create_knock_event_template` endpoint.
#[request]
pub struct Request {
    /// The room ID that should receive the knock.
    #[ruma_api(path)]
    pub room_id: RoomId,

    /// The user ID the knock event will be for.
    #[ruma_api(path)]
    pub user_id: UserId,

    /// The room versions the sending has support for.
    ///
    /// Defaults to `vec![RoomVersionId::V1]`.
    #[ruma_api(query)]
    pub ver: Vec<RoomVersionId>,
}

impl Request {
    /// Creates a `Request` with the given room ID and user ID.
    pub fn new(room_id: RoomId, user_id: UserId) -> Self {
        Self { room_id, user_id, ver: vec![RoomVersionId::V1] }
    }
}

impl Metadata for Request {
    const METHOD: http::Method = super::v1::Request::METHOD;
    const RATE_LIMITED: bool = super::v1::Request::RATE_LIMITED;
    type Authentication = <super::v1::Request as Metadata>::Authentication;
    type PathBuilder = <super::v1::Request as Metadata>::PathBuilder;
    const PATH_BUILDER: Self::PathBuilder = SinglePath::new(
        "/_matrix/federation/unstable/xyz.amorgan.knock/make_knock/{room_id}/{user_id}",
    );
}

impl From<super::v1::Request> for Request {
    fn from(value: super::v1::Request) -> Self {
        let super::v1::Request { room_id, user_id, ver } = value;
        Self { room_id, user_id, ver }
    }
}

impl From<Request> for super::v1::Request {
    fn from(value: Request) -> Self {
        let Request { room_id, user_id, ver } = value;
        Self { room_id, user_id, ver }
    }
}

/// Response type for the `create_knock_event_template` endpoint.
#[response]
pub struct Response {
    /// The version of the room where the server is trying to knock.
    pub room_version: RoomVersionId,

    /// An unsigned template event.
    ///
    /// May differ between room versions.
    pub event: Box<RawJsonValue>,
}

impl Response {
    /// Creates a new `Response` with the given room version ID and event.
    pub fn new(room_version: RoomVersionId, event: Box<RawJsonValue>) -> Self {
        Self { room_version, event }
    }
}

impl From<super::v1::Response> for Response {
    fn from(value: super::v1::Response) -> Self {
        let super::v1::Response { room_version, event } = value;
        Self { room_version, event }
    }
}

impl From<Response> for super::v1::Response {
    fn from(value: Response) -> Self {
        let Response { room_version, event } = value;
        Self { room_version, event }
    }
}
