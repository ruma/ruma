//! [GET /_matrix/client/r0/rooms/{roomId}/messages](https://matrix.org/docs/spec/client_server/r0.6.0#get-matrix-client-r0-rooms-roomid-messages)

use js_int::{uint, UInt};
use ruma_api::ruma_api;
use ruma_events::{AnyRoomEvent, AnyStateEvent};
use ruma_identifiers::RoomId;
use ruma_serde::Raw;
use serde::{Deserialize, Serialize};

use crate::r0::filter::{IncomingRoomEventFilter, RoomEventFilter};

ruma_api! {
    metadata: {
        description: "Get message events for a room.",
        method: GET,
        name: "get_message_events",
        path: "/_matrix/client/r0/rooms/:room_id/messages",
        rate_limited: false,
        authentication: AccessToken,
    }

    request: {
        /// The room to get events from.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The token to start returning events from.
        ///
        /// This token can be obtained from a
        /// prev_batch token returned for each room by the sync API, or from a start or end token
        /// returned by a previous request to this endpoint.
        #[ruma_api(query)]
        pub from: &'a str,

        /// The token to stop returning events at.
        ///
        /// This token can be obtained from a prev_batch
        /// token returned for each room by the sync endpoint, or from a start or end token returned
        /// by a previous request to this endpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub to: Option<&'a str>,

        /// The direction to return events from.
        #[ruma_api(query)]
        pub dir: Direction,

        /// The maximum number of events to return.
        ///
        /// Default: 10.
        #[ruma_api(query)]
        #[serde(default = "default_limit", skip_serializing_if = "is_default_limit")]
        pub limit: UInt,

        /// A RoomEventFilter to filter returned events with.
        #[ruma_api(query)]
        #[serde(
            with = "ruma_serde::json_string",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub filter: Option<RoomEventFilter<'a>>,
    }

    #[derive(Default)]
    response: {
        /// The token the pagination starts from.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub start: Option<String>,

        /// The token the pagination ends at.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub end: Option<String>,

        /// A list of room events.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub chunk: Vec<Raw<AnyRoomEvent>>,

        /// A list of state events relevant to showing the `chunk`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub state: Vec<Raw<AnyStateEvent>>,
    }

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given parameters.
    ///
    /// All other parameters will be defaulted.
    pub fn new(room_id: &'a RoomId, from: &'a str, dir: Direction) -> Self {
        Self { room_id, from, to: None, dir, limit: default_limit(), filter: None }
    }

    /// Creates a new `Request` with the given room ID and from token, and `dir` set to `Backward`.
    pub fn backward(room_id: &'a RoomId, from: &'a str) -> Self {
        Self::new(room_id, from, Direction::Backward)
    }

    /// Creates a new `Request` with the given room ID and from token, and `dir` set to `Forward`.
    pub fn forward(room_id: &'a RoomId, from: &'a str) -> Self {
        Self::new(room_id, from, Direction::Forward)
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Default::default()
    }
}

fn default_limit() -> UInt {
    uint!(10)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_limit(val: &UInt) -> bool {
    *val == default_limit()
}

/// The direction to return events from.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Direction {
    /// Return events backwards in time from the requested `from` token.
    #[serde(rename = "b")]
    Backward,

    /// Return events forwards in time from the requested `from` token.
    #[serde(rename = "f")]
    Forward,
}

#[cfg(test)]
mod tests {
    use super::{Direction, Request};

    use js_int::uint;
    use ruma_api::OutgoingRequest;
    use ruma_identifiers::room_id;

    use crate::r0::filter::{LazyLoadOptions, RoomEventFilter};

    #[test]
    fn serialize_some_room_event_filter() {
        let room_id = room_id!("!roomid:example.org");
        let rooms = &[room_id.clone()];
        let filter = RoomEventFilter {
            lazy_load_options: LazyLoadOptions::Enabled { include_redundant_members: true },
            rooms: Some(rooms),
            not_rooms: &["room".into(), "room2".into(), "room3".into()],
            not_types: &["type".into()],
            ..Default::default()
        };
        let req = Request {
            room_id: &room_id,
            from: "token",
            to: Some("token2"),
            dir: Direction::Backward,
            limit: uint!(0),
            filter: Some(filter),
        };

        let request: http::Request<Vec<u8>> =
            req.try_into_http_request("https://homeserver.tld", Some("auth_tok")).unwrap();
        assert_eq!(
            "from=token\
             &to=token2\
             &dir=b\
             &limit=0\
             &filter=%7B%22not_types%22%3A%5B%22type%22%5D%2C%22not_rooms%22%3A%5B%22room%22%2C%22\
              room2%22%2C%22room3%22%5D%2C%22rooms%22%3A%5B%22%21roomid%3Aexample.org%22%5D%2C%22\
              lazy_load_members%22%3Atrue%2C%22include_redundant_members%22%3Atrue%7D",
            request.uri().query().unwrap()
        );
    }

    #[test]
    fn serialize_none_room_event_filter() {
        let room_id = room_id!("!roomid:example.org");
        let req = Request {
            room_id: &room_id,
            from: "token",
            to: Some("token2"),
            dir: Direction::Backward,
            limit: uint!(0),
            filter: None,
        };

        let request =
            req.try_into_http_request("https://homeserver.tld", Some("auth_tok")).unwrap();
        assert_eq!("from=token&to=token2&dir=b&limit=0", request.uri().query().unwrap(),);
    }

    #[test]
    fn serialize_default_room_event_filter() {
        let room_id = room_id!("!roomid:example.org");
        let req = Request {
            room_id: &room_id,
            from: "token",
            to: Some("token2"),
            dir: Direction::Backward,
            limit: uint!(0),
            filter: Some(RoomEventFilter::default()),
        };

        let request: http::Request<Vec<u8>> =
            req.try_into_http_request("https://homeserver.tld", Some("auth_tok")).unwrap();
        assert_eq!(
            "from=token&to=token2&dir=b&limit=0&filter=%7B%7D",
            request.uri().query().unwrap(),
        );
    }
}
