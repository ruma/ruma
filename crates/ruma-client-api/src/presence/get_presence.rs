//! `GET /_matrix/client/*/presence/{userId}/status`
//!
//! Get presence status for this user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3presenceuseridstatus

    use std::time::Duration;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        presence::PresenceState,
        OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/presence/{user_id}/status",
            1.1 => "/_matrix/client/v3/presence/{user_id}/status",
        }
    };

    /// Request type for the `get_presence` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose presence state will be retrieved.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,
    }

    /// Response type for the `get_presence` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The state message for this user if one was set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status_msg: Option<String>,

        /// Whether or not the user is currently active.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub currently_active: Option<bool>,

        /// The length of time in milliseconds since an action was performed by the user.
        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pub last_active_ago: Option<Duration>,

        /// The user's presence state.
        pub presence: PresenceState,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given presence state.
        pub fn new(presence: PresenceState) -> Self {
            Self { presence, status_msg: None, currently_active: None, last_active_ago: None }
        }
    }
}
