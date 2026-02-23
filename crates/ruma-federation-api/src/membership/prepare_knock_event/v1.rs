//! `/v1/` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1make_knockroomiduserid

use ruma_common::{
    RoomId, RoomVersionId, UserId,
    api::{request, response},
    metadata,
};
use serde_json::value::RawValue as RawJsonValue;

use crate::authentication::ServerSignatures;

metadata! {
    method: GET,
    rate_limited: false,
    authentication: ServerSignatures,
    path: "/_matrix/federation/v1/make_knock/{room_id}/{user_id}",
}

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

impl Request {
    /// Creates a `Request` with the given room ID and user ID.
    pub fn new(room_id: RoomId, user_id: UserId) -> Self {
        Self { room_id, user_id, ver: vec![RoomVersionId::V1] }
    }
}

impl Response {
    /// Creates a new `Response` with the given room version ID and event.
    pub fn new(room_version: RoomVersionId, event: Box<RawJsonValue>) -> Self {
        Self { room_version, event }
    }
}
