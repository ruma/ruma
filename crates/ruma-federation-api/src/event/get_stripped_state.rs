//! `POST /_matrix/federation/*/stripped_state/{room_id}`
//!
//! Returns stripped state events in a room.

pub mod unstable {
    //! `/unstable/org.continuwuity.msc4560.stripped_state/` ([MSC4560])
    //!
    //! [MSC4560]: https://github.com/matrix-org/matrix-spec-proposals/pull/4560

    use std::collections::BTreeSet;

    use js_int::UInt;
    use ruma_common::{
        OwnedRoomId,
        api::{request, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::{AnyStrippedStateEvent, StateEventType};

    use crate::authentication::ServerSignatures;

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: ServerSignatures,
        path: "/_matrix/federation/unstable/org.continuwuity.msc4560.stripped_state/{room_id}",
    }

    /// Request type for the `get_stripped_state` endpoint.
    #[request]
    pub struct Request {
        /// The room to look up the state for.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// A list of state event types to fetch.
        ///
        /// This may only include event types listed in
        /// [`ruma_events::ALLOWED_STRIPPED_STATE_EVENTS`].
        ///
        /// Defaults to an empty `BTreeSet`.
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        pub state: BTreeSet<StateEventType>,

        /// Controls whether the response will include the number of members
        /// joined to the room.
        ///
        /// Defaults to `false`.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub num_joined_members: bool,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id, state: BTreeSet::new(), num_joined_members: false }
        }

        /// Returns whether this `Request` requests no data from the server.
        pub fn is_empty(&self) -> bool {
            let Self { room_id: _, state, num_joined_members } = &self;

            state.is_empty() && !*num_joined_members
        }
    }

    /// Response type for the `get_stripped_state` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// The requested stripped state events.
        pub state: Vec<Raw<AnyStrippedStateEvent>>,

        /// The number of members joined to the room, if `num_joined_members` was
        /// set to `true` in the request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub num_joined_members: Option<UInt>,
    }

    impl Response {
        /// Creates a new empty `Response`.
        pub fn new() -> Self {
            Self::default()
        }
    }
}
