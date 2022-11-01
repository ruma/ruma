//! `PUT /_matrix/client/*/presence/{userId}/status`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3presenceuseridstatus

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        presence::PresenceState,
        UserId,
    };

    const METADATA: Metadata = metadata! {
        description: "Set presence status for this user.",
        method: PUT,
        name: "set_presence",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/presence/:user_id/status",
            1.1 => "/_matrix/client/v3/presence/:user_id/status",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The user whose presence state will be updated.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The new presence state.
        pub presence: PresenceState,

        /// The status message to attach to this state.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status_msg: Option<&'a str>,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID and presence state.
        pub fn new(user_id: &'a UserId, presence: PresenceState) -> Self {
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
