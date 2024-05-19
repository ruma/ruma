//! `GET /_matrix/federation/*/make_join/{roomId}/{userId}`
//!
//! Send a request for a join event template to a resident server.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1make_joinroomiduserid

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
            1.0 => "/_matrix/federation/v1/make_join/{room_id}/{user_id}",
        }
    };

    /// Request type for the `create_join_event_template` endpoint.
    #[request]
    pub struct Request {
        /// The room ID that is about to be joined.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The user ID the join event will be for.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The room versions the sending server has support for.
        ///
        /// Defaults to `&[RoomVersionId::V1]`.
        #[ruma_api(query)]
        #[serde(default = "default_ver", skip_serializing_if = "is_default_ver")]
        pub ver: Vec<RoomVersionId>,
    }

    /// Response type for the `create_join_event_template` endpoint.
    #[response]
    pub struct Response {
        /// The version of the room where the server is trying to join.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_version: Option<RoomVersionId>,

        /// An unsigned template event.
        pub event: Box<RawJsonValue>,
    }

    fn default_ver() -> Vec<RoomVersionId> {
        vec![RoomVersionId::V1]
    }

    fn is_default_ver(ver: &[RoomVersionId]) -> bool {
        *ver == [RoomVersionId::V1]
    }

    impl Request {
        /// Creates a new `Request` with the given room id and user id.
        pub fn new(room_id: OwnedRoomId, user_id: OwnedUserId) -> Self {
            Self { room_id, user_id, ver: vec![RoomVersionId::V1] }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given template event.
        pub fn new(event: Box<RawJsonValue>) -> Self {
            Self { room_version: None, event }
        }
    }
}
