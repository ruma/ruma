//! `/unstable/xyz.amorgan.knock/` ([MSC])
//!
//! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2403

use ruma_common::{
    EventId, RoomId,
    api::{Metadata, path_builder::SinglePath, request, response},
};
use serde_json::value::RawValue as RawJsonValue;

use crate::membership::RawStrippedState;

/// Request type for the `send_knock` endpoint.
#[request]
pub struct Request {
    /// The room ID that should receive the knock.
    #[ruma_api(path)]
    pub room_id: RoomId,

    /// The event ID for the knock event.
    #[ruma_api(path)]
    pub event_id: EventId,

    /// The PDU.
    #[ruma_api(body)]
    pub pdu: Box<RawJsonValue>,
}

impl Request {
    /// Creates a new `Request` with the given room ID, event ID and knock event.
    pub fn new(room_id: RoomId, event_id: EventId, pdu: Box<RawJsonValue>) -> Self {
        Self { room_id, event_id, pdu }
    }
}

impl Metadata for Request {
    const METHOD: http::Method = super::v1::Request::METHOD;
    const RATE_LIMITED: bool = super::v1::Request::RATE_LIMITED;
    type Authentication = <super::v1::Request as Metadata>::Authentication;
    type PathBuilder = <super::v1::Request as Metadata>::PathBuilder;
    const PATH_BUILDER: Self::PathBuilder = SinglePath::new(
        "/_matrix/federation/unstable/xyz.amorgan.knock/send_knock/{room_id}/{event_id}",
    );
}

impl From<super::v1::Request> for Request {
    fn from(value: super::v1::Request) -> Self {
        let super::v1::Request { room_id, event_id, pdu } = value;
        Self { room_id, event_id, pdu }
    }
}

impl From<Request> for super::v1::Request {
    fn from(value: Request) -> Self {
        let Request { room_id, event_id, pdu } = value;
        Self { room_id, event_id, pdu }
    }
}

/// Response type for the `send_knock` endpoint.
#[response]
pub struct Response {
    /// State events providing public room metadata.
    pub knock_room_state: Vec<RawStrippedState>,
}

impl Response {
    /// Creates a new `Response` with the given public room metadata state events.
    pub fn new(knock_room_state: Vec<RawStrippedState>) -> Self {
        Self { knock_room_state }
    }
}

impl From<super::v1::Response> for Response {
    fn from(value: super::v1::Response) -> Self {
        let super::v1::Response { knock_room_state } = value;
        Self { knock_room_state }
    }
}

impl From<Response> for super::v1::Response {
    fn from(value: Response) -> Self {
        let Response { knock_room_state } = value;
        Self { knock_room_state }
    }
}
