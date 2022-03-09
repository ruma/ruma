//! `GET /_matrix/federation/*/make_join/{roomId}/{userId}`
//!
//! Endpoint to request a template for join events.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1make_joinroomiduserid

    use ruma_common::{api::ruma_api, RoomId, RoomVersionId, UserId};
    use serde_json::value::RawValue as RawJsonValue;

    ruma_api! {
        metadata: {
            description: "Send a request for a join event template to a resident server.",
            name: "create_join_event_template",
            method: GET,
            stable_path: "/_matrix/federation/v1/make_join/:room_id/:user_id",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.0,
        }

        request: {
            /// The room ID that is about to be joined.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The user ID the join event will be for.
            #[ruma_api(path)]
            pub user_id: &'a UserId,

            /// The room versions the sending server has support for.
            ///
            /// Defaults to `&[RoomVersionId::V1]`.
            #[ruma_api(query)]
            #[serde(default = "default_ver", skip_serializing_if = "is_default_ver")]
            pub ver: &'a [RoomVersionId],
        }

        response: {
            /// The version of the room where the server is trying to join.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub room_version: Option<RoomVersionId>,

            /// An unsigned template event.
            pub event: Box<RawJsonValue>,
        }
    }

    fn default_ver() -> Vec<RoomVersionId> {
        vec![RoomVersionId::V1]
    }

    fn is_default_ver(ver: &&[RoomVersionId]) -> bool {
        **ver == [RoomVersionId::V1]
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room id and user id.
        pub fn new(room_id: &'a RoomId, user_id: &'a UserId) -> Self {
            Self { room_id, user_id, ver: &[RoomVersionId::V1] }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given template event.
        pub fn new(event: Box<RawJsonValue>) -> Self {
            Self { room_version: None, event }
        }
    }
}
