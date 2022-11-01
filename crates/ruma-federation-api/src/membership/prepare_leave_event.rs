//! `GET /_matrix/federation/*/make_leave/{roomId}/{userId}`
//!
//! Endpoint to asks the receiving server to return information that the sending server will need
//! to prepare a leave event to get out of the room.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/server-server-api/#get_matrixfederationv1make_leaveroomiduserid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, RoomId, RoomVersionId, UserId,
    };
    use serde_json::value::RawValue as RawJsonValue;

    const METADATA: Metadata = metadata! {
        description: "Asks the receiving server to return information that the sending server will need to prepare a leave event to get out of the room.",
        method: GET,
        name: "get_leave_event",
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            1.0 => "/_matrix/federation/v1/make_leave/:room_id/:user_id",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The room ID that is about to be left.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The user ID the leave event will be for.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

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

    impl<'a> Request<'a> {
        /// Creates a new `Request` with:
        /// * the room ID that is about to be left.
        /// * the user ID the leave event will be for.
        pub fn new(room_id: &'a RoomId, user_id: &'a UserId) -> Self {
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
