//! `GET /_matrix/client/*/profile/{userId}/displayname`
//!
//! Get the display name of a user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3profileuseriddisplayname

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/profile/{user_id}/displayname",
            1.1 => "/_matrix/client/v3/profile/{user_id}/displayname",
        }
    };

    /// Request type for the `get_display_name` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user whose display name will be retrieved.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,
    }

    /// Response type for the `get_display_name` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// The user's display name, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given display name.
        pub fn new(displayname: Option<String>) -> Self {
            Self { displayname }
        }
    }
}
