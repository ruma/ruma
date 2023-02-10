//! `GET /_matrix/client/*/keys/changes`
//!
//! Gets a list of users who have updated their device identity keys since a previous sync token.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3keyschanges

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/keys/changes",
            1.1 => "/_matrix/client/v3/keys/changes",
        }
    };

    /// Request type for the `get_key_changes` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The desired start point of the list.
        ///
        /// Should be the next_batch field from a response to an earlier call to /sync.
        #[ruma_api(query)]
        pub from: String,

        /// The desired end point of the list.
        ///
        /// Should be the next_batch field from a recent call to /sync - typically the most recent
        /// such call.
        #[ruma_api(query)]
        pub to: String,
    }

    /// Response type for the `get_key_changes` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The Matrix User IDs of all users who updated their device identity keys.
        pub changed: Vec<OwnedUserId>,

        /// The Matrix User IDs of all users who may have left all the end-to-end
        /// encrypted rooms they previously shared with the user.
        pub left: Vec<OwnedUserId>,
    }

    impl Request {
        /// Creates a new `Request` with the given start and end points.
        pub fn new(from: String, to: String) -> Self {
            Self { from, to }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given changed and left user ID lists.
        pub fn new(changed: Vec<OwnedUserId>, left: Vec<OwnedUserId>) -> Self {
            Self { changed, left }
        }
    }
}
