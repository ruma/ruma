//! `PUT /_matrix/client/*/presence/{userId}/status`
//!
//! Set presence status for this user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3presenceuseridstatus

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        presence::PresenceState,
        OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/presence/{user_id}/status",
            1.1 => "/_matrix/client/v3/presence/{user_id}/status",
        }
    };

    /// Request type for the `set_presence` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose presence state will be updated.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// The new presence state.
        pub presence: PresenceState,

        /// The status message to attach to this state.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status_msg: Option<String>,
    }

    /// Response type for the `set_presence` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID and presence state.
        pub fn new(user_id: OwnedUserId, presence: PresenceState) -> Self {
            Self { user_id, presence, status_msg: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
