//! `POST /_matrix/client/*/stripped_state/{room_id_or_alias}`
//!
//! Returns stripped state events in a room.

pub mod unstable {
    //! `/unstable/org.continuwuity.msc4560.stripped_state/` ([MSC4560])
    //!
    //! [MSC4560]: https://github.com/matrix-org/matrix-spec-proposals/pull/4560

    use std::collections::BTreeSet;

    use js_int::UInt;
    use ruma_common::{
        OwnedRoomOrAliasId, OwnedServerName,
        api::{auth_scheme::AccessTokenOptional, request, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::{AnyStrippedStateEvent, StateEventType};

    /// The maximum number of vias that may be included in the request.
    ///
    /// If more than this many vias are included, `M_INVALID_PARAM` will be returned.
    pub const MAX_VIAS: usize = 32;

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessTokenOptional,
        history: {
            unstable("org.continuwuity.msc4560") => "/_matrix/client/unstable/org.continuwuity.msc4560.stripped_state/{room_id_or_alias}",
        }
    }

    /// Request type for the `get_stripped_state` endpoint.
    #[request]
    pub struct Request {
        /// The room to look up the state for.
        #[ruma_api(path)]
        pub room_id_or_alias: OwnedRoomOrAliasId,

        /// A list of servers the homeserver should attempt to use to fetch the room's state,
        /// if it is not a resident of the room.
        ///
        /// Defaults to an empty `BTreeSet`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ruma_api(query)]
        pub via: Vec<OwnedServerName>,

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
        /// Creates a new `Request` with the given room ID or alias and the given vias.
        pub fn new(room_id_or_alias: OwnedRoomOrAliasId, via: Vec<OwnedServerName>) -> Self {
            Self { room_id_or_alias, via, state: BTreeSet::new(), num_joined_members: false }
        }

        /// Returns whether this `Request` requests no data from the server.
        pub fn is_empty(&self) -> bool {
            let Self { room_id_or_alias: _, via: _, state, num_joined_members } = &self;

            state.is_empty() && !*num_joined_members
        }
    }

    /// Response type for the `get_stripped_state` endpoint.
    #[response]
    pub struct Response {
        /// The requested stripped state events.
        pub state: Vec<Raw<AnyStrippedStateEvent>>,

        /// The number of members joined to the room, if `num_joined_members` was
        /// set to `true` in the request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub num_joined_members: Option<UInt>,

        /// The server that provided the data included in this response.
        ///
        /// This will be
        /// the local server's server name if it was able to generate the stripped state
        /// locally.
        pub via: OwnedServerName,
    }

    impl Response {
        /// Creates a new `Response` with the given via.
        pub fn new(via: OwnedServerName) -> Self {
            Self { state: vec![], num_joined_members: None, via }
        }
    }
}
