//! `GET /_matrix/federation/*/make_leave/{roomId}/{userId}`
//!
//! Asks the receiving server to return information that the sending server will need to prepare a
//! leave event to get out of the room.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1make_leaveroomiduserid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedUserId, RoomVersionId,
    };
    use serde_json::value::RawValue as RawJsonValue;

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/make_leave/{room_id}/{user_id}",
        }
    };

    /// Request type for the `get_leave_event` endpoint.
    #[request]
    pub struct Request {
        /// The room ID that is about to be left.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The user ID the leave event will be for.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,
    }

    /// Response type for the `get_leave_event` endpoint.
    #[response]
    pub struct Response {
        /// The version of the room where the server is trying to leave.
        ///
        /// If not provided, the room version is assumed to be either "1" or "2".
        pub room_version: Option<RoomVersionId>,

        /// An unsigned template event.
        ///
        /// Note that events have a different format depending on the room version - check the room
        /// version specification for precise event formats.
        pub event: Box<RawJsonValue>,
    }

    impl Request {
        /// Creates a new `Request` with:
        /// * the room ID that is about to be left.
        /// * the user ID the leave event will be for.
        pub fn new(room_id: OwnedRoomId, user_id: OwnedUserId) -> Self {
            Self { room_id, user_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with:
        /// * the version of the room where the server is trying to leave.
        /// * an unsigned template event.
        pub fn new(room_version: Option<RoomVersionId>, event: Box<RawJsonValue>) -> Self {
            Self { room_version, event }
        }
    }
}
