//! `GET /_matrix/federation/*/hierarchy/{roomId}`
//!
//! Get the space tree in a depth-first manner to locate child rooms of a given space.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#get_matrixfederationv1hierarchyroomid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    use crate::space::{SpaceHierarchyChildSummary, SpaceHierarchyParentSummary};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        history: {
            unstable => "/_matrix/federation/unstable/org.matrix.msc2946/hierarchy/{room_id}",
            1.2 => "/_matrix/federation/v1/hierarchy/{room_id}",
        }
    };

    /// Request type for the `hierarchy` endpoint.
    #[request]
    pub struct Request {
        /// The room ID of the space to get a hierarchy for.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// Whether or not the server should only consider suggested rooms.
        ///
        /// Suggested rooms are annotated in their `m.space.child` event contents.
        #[ruma_api(query)]
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub suggested_only: bool,
    }

    /// Response type for the `hierarchy` endpoint.
    #[response]
    pub struct Response {
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

    impl Request {
        /// Creates a `Request` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
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
