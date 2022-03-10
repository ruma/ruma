//! `GET /_matrix/client/*/rooms/{roomId}/hierarchy`

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv1roomsroomidhierarchy

    use js_int::UInt;
    use ruma_common::{api::ruma_api, RoomId};

    use crate::space::SpaceHierarchyRoomsChunk;

    ruma_api! {
        metadata: {
            description: "Paginates over the space tree in a depth-first manner to locate child rooms of a given space.",
            method: GET,
            name: "hierarchy",
            unstable_path: "/_matrix/client/unstable/org.matrix.msc2946/rooms/:room_id/hierarchy",
            stable_path: "/_matrix/client/v1/rooms/:room_id/hierarchy",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.2,
        }

        request: {
            /// The room ID of the space to get a hierarchy for.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// A pagination token from a previous result.
            ///
            /// If specified, `max_depth` and `suggested_only` cannot be changed from the first request.
            #[ruma_api(query)]
            pub from: Option<&'a str>,

            /// The maximum number of rooms to include per response.
            #[ruma_api(query)]
            pub limit: Option<UInt>,

            /// How far to go into the space.
            ///
            /// When reached, no further child rooms will be returned.
            #[ruma_api(query)]
            pub max_depth: Option<UInt>,

            /// Whether or not the server should only consider suggested rooms.
            ///
            /// Suggested rooms are annotated in their `m.space.child` event contents.
            ///
            /// Defaults to `false`.
            #[ruma_api(query)]
            #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
            pub suggested_only: bool,
        }

        #[derive(Default)]
        response: {
            /// A token to supply to from to keep paginating the responses.
            ///
            /// Not present when there are no further results.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub next_batch: Option<String>,

            /// A paginated chunk of the space children.
            pub rooms: Vec<SpaceHierarchyRoomsChunk>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: &'a RoomId) -> Self {
            Self { room_id, from: None, limit: None, max_depth: None, suggested_only: false }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Default::default()
        }
    }
}
