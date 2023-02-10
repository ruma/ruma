//! `POST /_matrix/client/*/pushers/set`
//!
//! This endpoint allows the creation, modification and deletion of pushers for this user ID.

mod set_pusher_serde;

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3pushersset

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };
    use serde::Serialize;

    use crate::push::{Pusher, PusherIds};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/pushers/set",
            1.1 => "/_matrix/client/v3/pushers/set",
        }
    };

    /// Request type for the `set_pusher` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The action to take.
        #[ruma_api(body)]
        pub action: PusherAction,
    }

    /// Response type for the `set_pusher` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

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
    #[derive(Clone, Debug, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct PusherPostData {
        /// The pusher to configure.
        #[serde(flatten)]
        pub pusher: Pusher,

        /// Controls if another pusher with the same pushkey and app id should be created, if there
        /// are already others for other users.
        ///
        /// Defaults to `false`. See the spec for more details.
        #[serde(skip_serializing_if = "ruma_common::serde::is_default")]
        pub append: bool,
    }
}
