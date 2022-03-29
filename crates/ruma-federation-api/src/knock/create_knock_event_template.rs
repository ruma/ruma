//! `GET /_matrix/federation/*/make_knock/{roomId}/{userId}`
//!
//! Endpoint to query information to prepare a knock event.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1make_knockroomiduserid

    use ruma_common::{api::ruma_api, RoomId, RoomVersionId, UserId};
    use serde_json::value::RawValue as RawJsonValue;

    ruma_api! {
        metadata: {
            description: "Send a request for a knock event template to a resident server.",
            name: "create_knock_event_template",
            method: GET,
            unstable_path: "/_matrix/federation/unstable/xyz.amorgan.knock/make_knock/:room_id/:user_id",
            stable_path: "/_matrix/federation/v1/make_knock/:room_id/:user_id",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.1,
        }

        request: {
            /// The room ID that should receive the knock.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The user ID the knock event will be for.
            #[ruma_api(path)]
            pub user_id: &'a UserId,

            /// The room versions the sending has support for.
            ///
            /// Defaults to `&[RoomVersionId::V1]`.
            #[ruma_api(query)]
            pub ver: &'a [RoomVersionId],
        }

        response: {
            /// The version of the room where the server is trying to knock.
            pub room_version: RoomVersionId,

            /// An unsigned template event.
            ///
            /// May differ between room versions.
            pub event: Box<RawJsonValue>,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a `Request` with the given room ID and user ID.
        pub fn new(room_id: &'a RoomId, user_id: &'a UserId) -> Self {
            Self { room_id, user_id, ver: &[RoomVersionId::V1] }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room version ID and event.
        pub fn new(room_version: RoomVersionId, event: Box<RawJsonValue>) -> Self {
            Self { room_version, event }
        }
    }
}
