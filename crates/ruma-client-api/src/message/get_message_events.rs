//! `GET /_matrix/client/*/rooms/{roomId}/messages`
//!
//! Get message events for a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#get_matrixclientv3roomsroomidmessages

    use js_int::{uint, UInt};
    use ruma_common::{
        api::{request, response, Metadata},
        events::{AnyStateEvent, AnyTimelineEvent},
        metadata,
        serde::Raw,
        RoomId,
    };

    use crate::{
        filter::{IncomingRoomEventFilter, RoomEventFilter},
        Direction,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/:room_id/messages",
            1.1 => "/_matrix/client/v3/rooms/:room_id/messages",
        }
    };

    /// Request type for the `get_message_events` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The room to get events from.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The token to start returning events from.
        ///
        /// This token can be obtained from a `prev_batch` token returned for each room by the
        /// sync endpoint, or from a `start` or `end` token returned by a previous request to
        /// this endpoint.
        ///
        /// If this is `None`, the server will return messages from the start or end of the
        /// history visible to the user, depending on the value of [`dir`][Self::dir].
        #[ruma_api(query)]
        pub from: Option<&'a str>,

        /// The token to stop returning events at.
        ///
        /// This token can be obtained from a `prev_batch` token returned for each room by the
        /// sync endpoint, or from a `start` or `end` token returned by a previous request to
        /// this endpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub to: Option<&'a str>,

        /// The direction to return events from.
        #[ruma_api(query)]
        pub dir: Direction,

        /// The maximum number of events to return.
        ///
        /// Default: `10`.
        #[ruma_api(query)]
        #[serde(default = "default_limit", skip_serializing_if = "is_default_limit")]
        pub limit: UInt,

        /// A [`RoomEventFilter`] to filter returned events with.
        #[ruma_api(query)]
        #[serde(
            with = "ruma_common::serde::json_string",
            default,
            skip_serializing_if = "RoomEventFilter::is_empty"
        )]
        pub filter: RoomEventFilter<'a>,
    }

    /// Response type for the `get_message_events` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// The token the pagination starts from.
        pub start: String,

        /// The token the pagination ends at.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub end: Option<String>,

        /// A list of room events.
        #[serde(default)]
        pub chunk: Vec<Raw<AnyTimelineEvent>>,

        /// A list of state events relevant to showing the `chunk`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub state: Vec<Raw<AnyStateEvent>>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID and direction.
        ///
        /// All other parameters will be defaulted.
        pub fn new(room_id: &'a RoomId, dir: Direction) -> Self {
            Self {
                room_id,
                from: None,
                to: None,
                dir,
                limit: default_limit(),
                filter: RoomEventFilter::default(),
            }
        }

        /// Creates a new `Request` with the given room ID and `dir` set to `Backward`.
        ///
        /// If the returned request is sent without `from` being set, pagination will start at the
        /// end of (the accessible part of) the room timeline.
        ///
        /// # Example
        ///
        /// ```rust
        /// # use ruma_client_api::message::get_message_events;
        /// # let room_id = ruma_common::room_id!("!a:example.org");
        /// # let token = "prev_batch token";
        /// let request = get_message_events::v3::Request::backward(room_id).from(token);
        /// ```
        pub fn backward(room_id: &'a RoomId) -> Self {
            Self::new(room_id, Direction::Backward)
        }

        /// Creates a new `Request` with the given room ID and `dir` set to `Forward`.
        ///
        /// If the returned request is sent without `from` being set, pagination will start at the
        /// beginning of (the accessible part of) the room timeline.
        ///
        /// # Example
        ///
        /// ```rust
        /// # use ruma_client_api::message::get_message_events;
        /// # let room_id = ruma_common::room_id!("!a:example.org");
        /// # let token = "end token";
        /// let request = get_message_events::v3::Request::forward(room_id).from(token);
        /// ```
        pub fn forward(room_id: &'a RoomId) -> Self {
            Self::new(room_id, Direction::Forward)
        }

        /// Creates a new `Request` from `self` with the `from` field set to the given value.
        ///
        /// Since the field is public, you can also assign to it directly. This method merely acts
        /// as a shorthand for that, because it is very common to set this field.
        pub fn from(self, from: impl Into<Option<&'a str>>) -> Self {
            Self { from: from.into(), ..self }
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

    #[cfg(all(test, feature = "client"))]
    mod tests {
        use js_int::uint;
        use ruma_common::{
            api::{MatrixVersion, OutgoingRequest, SendAccessToken},
            room_id,
        };

        use super::Request;
        use crate::{
            filter::{LazyLoadOptions, RoomEventFilter},
            Direction,
        };

        #[test]
        fn serialize_some_room_event_filter() {
            let room_id = room_id!("!roomid:example.org");
            let rooms = &[room_id.to_owned()];
            let filter = RoomEventFilter {
                lazy_load_options: LazyLoadOptions::Enabled { include_redundant_members: true },
                rooms: Some(rooms),
                not_rooms: &[
                    room_id!("!room:example.org").to_owned(),
                    room_id!("!room2:example.org").to_owned(),
                    room_id!("!room3:example.org").to_owned(),
                ],
                not_types: &["type".into()],
                ..Default::default()
            };
            let req = Request {
                room_id,
                from: Some("token"),
                to: Some("token2"),
                dir: Direction::Backward,
                limit: uint!(0),
                filter,
            };

            let request: http::Request<Vec<u8>> = req
                .try_into_http_request(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    &[MatrixVersion::V1_1],
                )
                .unwrap();
            assert_eq!(
            "from=token\
             &to=token2\
             &dir=b\
             &limit=0\
             &filter=%7B%22not_types%22%3A%5B%22type%22%5D%2C%22not_rooms%22%3A%5B%22%21room%3Aexample.org%22%2C%22%21room2%3Aexample.org%22%2C%22%21room3%3Aexample.org%22%5D%2C%22rooms%22%3A%5B%22%21roomid%3Aexample.org%22%5D%2C%22lazy_load_members%22%3Atrue%2C%22include_redundant_members%22%3Atrue%7D",
            request.uri().query().unwrap()
        );
        }

        #[test]
        fn serialize_default_room_event_filter() {
            let room_id = room_id!("!roomid:example.org");
            let req = Request {
                room_id,
                from: Some("token"),
                to: Some("token2"),
                dir: Direction::Backward,
                limit: uint!(0),
                filter: RoomEventFilter::default(),
            };

            let request = req
                .try_into_http_request::<Vec<u8>>(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    &[MatrixVersion::V1_1],
                )
                .unwrap();
            assert_eq!("from=token&to=token2&dir=b&limit=0", request.uri().query().unwrap(),);
        }
    }
}
