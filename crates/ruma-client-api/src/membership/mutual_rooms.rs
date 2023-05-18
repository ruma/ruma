//! `GET /_matrix/client/*/user/mutual_rooms/{user_id}`
//!
//! Get mutual rooms with another user.

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/blob/hs/shared-rooms/proposals/2666-get-rooms-in-common.md

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/uk.half-shot.msc2666/user/mutual_rooms",
        }
    };

    /// Request type for the `mutual_rooms` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The user to search mutual rooms for.
        #[ruma_api(query)]
        pub user_id: OwnedUserId,

        /// An opaque string, used to paginate.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub batch_token: Option<String>,
    }

    /// Response type for the `mutual_rooms` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A list of rooms the user is in together with the authenticated user.
        pub joined: Vec<OwnedRoomId>,

        /// An opaque string, returned when the server paginates this response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch_token: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: OwnedUserId, batch_token: Option<String>) -> Self {
            Self { user_id, batch_token }
        }
    }

    impl Response {
        /// Creates a `Response` with the given room ids.
        pub fn new(joined: Vec<OwnedRoomId>, next_batch_token: Option<String>) -> Self {
            Self { joined, next_batch_token }
        }
    }
}
