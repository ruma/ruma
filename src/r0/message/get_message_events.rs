//! [GET /_matrix/client/r0/rooms/{roomId}/messages](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-rooms-roomid-messages)

use js_int::UInt;
use ruma_api::ruma_api;
use ruma_events::{collections::all::RoomEvent, EventResult};
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

use crate::r0::filter::RoomEventFilter;

ruma_api! {
    metadata {
        description: "Get message events for a room.",
        method: GET,
        name: "get_message_events",
        path: "/_matrix/client/r0/rooms/:room_id/messages",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// The room to get events from.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The token to start returning events from.
        ///
        /// This token can be obtained from a
        /// prev_batch token returned for each room by the sync API, or from a start or end token
        /// returned by a previous request to this endpoint.
        #[ruma_api(query)]
        pub from: String,
        /// The token to stop returning events at.
        ///
        /// This token can be obtained from a prev_batch
        /// token returned for each room by the sync endpoint, or from a start or end token returned
        /// by a previous request to this endpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub to: Option<String>,
        /// The direction to return events from.
        #[ruma_api(query)]
        pub dir: Direction,
        /// The maximum number of events to return.
        ///
        /// Default: 10.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub limit: Option<UInt>,
        /// A RoomEventFilter to filter returned events with.
        #[ruma_api(query)]
        #[serde(
            with = "crate::serde::json_string",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub filter: Option<RoomEventFilter>,
    }

    response {
        /// The token the pagination starts from.
        pub start: String,
        /// A list of room events.
        #[wrap_incoming(RoomEvent with EventResult)]
        pub chunk: Vec<RoomEvent>,
        /// The token the pagination ends at.
        pub end: String,
    }

    error: crate::Error
}

/// The direction to return events from.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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

    use std::convert::{TryFrom, TryInto};

    use ruma_identifiers::RoomId;

    use crate::r0::filter::{LazyLoadOptions, RoomEventFilter};

    #[test]
    fn test_serialize_some_room_event_filter() {
        let room_id = RoomId::try_from("!roomid:example.org").unwrap();
        let filter = RoomEventFilter {
            lazy_load_options: LazyLoadOptions::Enabled {
                include_redundant_members: true,
            },
            rooms: Some(vec![room_id.clone()]),
            not_rooms: vec!["room".into(), "room2".into(), "room3".into()],
            not_types: vec!["type".into()],
            ..Default::default()
        };
        let req = Request {
            room_id,
            from: "token".into(),
            to: Some("token2".into()),
            dir: Direction::Backward,
            limit: Some(js_int::UInt::min_value()),
            filter: Some(filter),
        };

        let request: http::Request<Vec<u8>> = req.try_into().unwrap();
        assert_eq!(
            "from=token&to=token2&dir=b&limit=0&filter=%7B%22not_types%22%3A%5B%22type%22%5D%2C%22not_rooms%22%3A%5B%22room%22%2C%22room2%22%2C%22room3%22%5D%2C%22rooms%22%3A%5B%22%21roomid%3Aexample.org%22%5D%2C%22lazy_load_members%22%3Atrue%2C%22include_redundant_members%22%3Atrue%7D",
            request.uri().query().unwrap()
        );
    }

    #[test]
    fn test_serialize_none_room_event_filter() {
        let room_id = RoomId::try_from("!roomid:example.org").unwrap();
        let req = Request {
            room_id,
            from: "token".into(),
            to: Some("token2".into()),
            dir: Direction::Backward,
            limit: Some(js_int::UInt::min_value()),
            filter: None,
        };

        let request: http::Request<Vec<u8>> = req.try_into().unwrap();
        assert_eq!(
            "from=token&to=token2&dir=b&limit=0",
            request.uri().query().unwrap(),
        );
    }

    #[test]
    fn test_serialize_default_room_event_filter() {
        let room_id = RoomId::try_from("!roomid:example.org").unwrap();
        let req = Request {
            room_id,
            from: "token".into(),
            to: Some("token2".into()),
            dir: Direction::Backward,
            limit: Some(js_int::UInt::min_value()),
            filter: Some(RoomEventFilter::default()),
        };

        let request: http::Request<Vec<u8>> = req.try_into().unwrap();
        assert_eq!(
            "from=token&to=token2&dir=b&limit=0&filter=%7B%7D",
            request.uri().query().unwrap(),
        );
    }
}
