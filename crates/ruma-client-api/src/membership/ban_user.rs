//! `POST /_matrix/client/*/rooms/{roomId}/ban`
//!
//! Ban a user from a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/client-server-api/#post_matrixclientv3roomsroomidban

    use ruma_common::{
        OwnedRoomId, OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/ban",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/ban",
        }
    }

    /// Request type for the `ban_user` endpoint.
    #[request]
    pub struct Request {
        /// The room to kick the user from.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The user to ban.
        pub user_id: OwnedUserId,

        /// The reason for banning the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,

        /// A flag indicating whether all the user's events should be immediately redacted.
        ///
        /// This uses the unstable prefix defined in [MSC4293].
        ///
        /// [MSC4293]: https://github.com/matrix-org/matrix-spec-proposals/pull/4293
        #[cfg(feature = "unstable-msc4293")]
        #[serde(
            rename = "org.matrix.msc4293.redact_events",
            skip_serializing_if = "ruma_common::serde::is_default"
        )]
        pub redact_events: bool,
    }

    /// Response type for the `ban_user` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room id and room id.
        pub fn new(room_id: OwnedRoomId, user_id: OwnedUserId) -> Self {
            Self {
                room_id,
                user_id,
                reason: None,
                #[cfg(feature = "unstable-msc4293")]
                redact_events: false,
            }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
