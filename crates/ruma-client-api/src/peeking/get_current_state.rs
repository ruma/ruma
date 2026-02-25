//! `GET /_matrix/client/*/rooms/{roomId}/initialSync`
//!
//! Get a copy of the current state and the most recent messages in a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3roomsroomidinitialsync

    use ruma_common::{
        RoomId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::{
        AnyRoomAccountDataEvent, AnyStateEvent, AnyTimelineEvent, room::member::MembershipState,
    };
    use serde::{Deserialize, Serialize};

    use crate::room::Visibility;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/initialSync",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/initialSync",
        }
    }

    /// Request type for the `get_current_state` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room to get the data of.
        #[ruma_api(path)]
        pub room_id: RoomId,
    }

    impl Request {
        /// Creates a `Request` for the given room.
        pub fn new(room_id: RoomId) -> Self {
            Self { room_id }
        }
    }

    /// Response type for the `get_current_state` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The private data that this user has attached to this room.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub account_data: Vec<Raw<AnyRoomAccountDataEvent>>,

        /// The user’s membership state in this room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub membership: Option<MembershipState>,

        /// The pagination chunk for this room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub messages: Option<PaginationChunk>,

        /// The room ID for this room.
        pub room_id: RoomId,

        /// The state of the room.
        ///
        /// If the user is a member of the room this will be the current state of the room as a
        /// list of events.
        ///
        /// If the user has left the room this will be the state of the room when they left it.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub state: Vec<Raw<AnyStateEvent>>,

        /// Whether this room is visible to the `/publicRooms` API or not.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub visibility: Option<Visibility>,
    }

    impl Response {
        /// Creates a `Response` for the given room.
        pub fn new(room_id: RoomId) -> Self {
            Self {
                room_id,
                account_data: Vec::new(),
                membership: None,
                messages: None,
                state: Vec::new(),
                visibility: None,
            }
        }
    }

    /// A paginated chunk of messages from the room's timeline.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct PaginationChunk {
        /// An array of events.
        ///
        /// If the user is a member of the room this will be a list of the most recent messages for
        /// this room.
        ///
        /// If the user has left the room this will be the messages that preceded them leaving.
        pub chunk: Vec<Raw<AnyTimelineEvent>>,

        ///  A token which correlates to the end of chunk.
        ///
        /// Can be passed to [`listen_to_new_events`] to listen to new events and to
        /// [`get_message_events`] to retrieve later events.
        ///
        /// [`listen_to_new_events`]: crate::peeking::listen_to_new_events
        /// [`get_message_events`]: crate::message::get_message_events
        pub end: String,

        /// A token which correlates to the start of chunk. Can be passed to [`get_message_events`]
        /// to retrieve earlier events.
        ///
        /// If no earlier events are available, this property may be omitted from the response.
        ///
        /// [`get_message_events`]: crate::message::get_message_events
        #[serde(skip_serializing_if = "Option::is_none")]
        pub start: Option<String>,
    }

    impl PaginationChunk {
        /// Construct a new `PaginationChunk` with the given events and end token.
        pub fn new(chunk: Vec<Raw<AnyTimelineEvent>>, end: String) -> Self {
            Self { chunk, end, start: None }
        }
    }
}
