//! `GET /_matrix/federation/*/hierarchy/{roomId}`
//!
//! Endpoint to get the children of a given space.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/server-server-api/#get_matrixfederationv1hierarchyroomid

    use ruma_common::{api::ruma_api, OwnedRoomId, RoomId};

    use crate::space::{SpaceHierarchyChildSummary, SpaceHierarchyParentSummary};

    ruma_api! {
        metadata: {
            description: "Get the space tree in a depth-first manner to locate child rooms of a given space.",
            name: "hierarchy",
            method: GET,
            unstable_path: "/_matrix/federation/unstable/org.matrix.msc2946/hierarchy/:room_id",
            stable_path: "/_matrix/federation/v1/hierarchy/:room_id",
            rate_limited: false,
            authentication: ServerSignatures,
            added: 1.2,
        }

        request: {
            /// The room ID of the space to get a hierarchy for.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// Whether or not the server should only consider suggested rooms.
            ///
            /// Suggested rooms are annotated in their `m.space.child` event contents.
            #[ruma_api(query)]
            #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
            pub suggested_only: bool,
        }

        response: {
            /// A summary of the space’s children.
            ///
            /// Rooms which the requesting server cannot peek/join will be excluded.
            pub children: Vec<SpaceHierarchyChildSummary>,

            /// The list of room IDs the requesting server doesn’t have a viable way to peek/join.
            ///
            /// Rooms which the responding server cannot provide details on will be outright
            /// excluded from the response instead.
            pub inaccessible_children: Vec<OwnedRoomId>,

            /// A summary of the requested room.
            pub room: SpaceHierarchyParentSummary,
        }
    }

    impl<'a> Request<'a> {
        /// Creates a `Request` with the given room ID.
        pub fn new(room_id: &'a RoomId) -> Self {
            Self { room_id, suggested_only: false }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given room summary.
        pub fn new(room_summary: SpaceHierarchyParentSummary) -> Self {
            Self { children: Vec::new(), inaccessible_children: Vec::new(), room: room_summary }
        }
    }
}
