//! `POST /_matrix/client/*/pushers/set`

mod pusher_action_serde;

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3pushersset

    use ruma_common::api::ruma_api;
    use serde::{Deserialize, Serialize};

    use crate::push::{Pusher, PusherIds};

    ruma_api! {
        metadata: {
            description: "This endpoint allows the creation, modification and deletion of pushers for this user ID.",
            method: POST,
            name: "set_pusher",
            r0_path: "/_matrix/client/r0/pushers/set",
            stable_path: "/_matrix/client/v3/pushers/set",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The action to take.
            #[ruma_api(body)]
            pub action: PusherAction,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl Request {
        /// Creates a new `Request` for the given action.
        pub fn new(action: PusherAction) -> Self {
            Self { action }
        }

        /// Creates a new `Request` to create or update the given pusher.
        pub fn post(pusher: Pusher) -> Self {
            Self::new(PusherAction::Post(PusherPostData { pusher, append: false }))
        }

        /// Creates a new `Request` to delete the pusher identified by the given IDs.
        pub fn delete(ids: PusherIds) -> Self {
            Self::new(PusherAction::Delete(ids))
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    /// The action to take for the pusher.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub enum PusherAction {
        /// Create or update the given pusher.
        Post(PusherPostData),

        /// Delete the pusher identified by the given IDs.
        Delete(PusherIds),
    }

    /// Data necessary to create or update a pusher.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct PusherPostData {
        /// The pusher to configure.
        #[serde(flatten)]
        pub pusher: Pusher,

        /// Controls if another pusher with the same pushkey and app id should be created, if there
        /// are already others for other users.
        ///
        /// Defaults to `false`. See the spec for more details.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub append: bool,
    }
}
