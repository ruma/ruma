//! `GET /_matrix/client/*/mutual_rooms`
//!
//! Get mutual rooms with another user.

pub mod unstable {
    //! `GET /_matrix/client/unstable/uk.half-shot.msc2666/user/mutual_rooms` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/2666

    use ruma_common::{
        OwnedUserId, RoomId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("uk.half-shot.msc2666.query_mutual_rooms") => "/_matrix/client/unstable/uk.half-shot.msc2666/user/mutual_rooms",
        }
    }

    /// Request type for the `mutual_rooms` endpoint.
    #[request]
    pub struct Request {
        /// The user to search mutual rooms for.
        #[ruma_api(query)]
        pub user_id: OwnedUserId,

        /// The `next_batch` token returned from a previous response, to get the next batch of
        /// rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub from: Option<String>,
    }

    /// Response type for the `mutual_rooms` endpoint.
    #[response]
    pub struct Response {
        /// A list of rooms the user is in together with the authenticated user.
        pub joined: Vec<RoomId>,

        /// An opaque string, returned when the server paginates this response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, from: None }
        }

        /// Creates a new `Request` with the given user id, together with a batch token.
        pub fn with_batch_token(user_id: OwnedUserId, token: String) -> Self {
            Self { user_id, from: Some(token) }
        }
    }

    impl Response {
        /// Creates a `Response` with the given room ids.
        pub fn new(joined: Vec<RoomId>) -> Self {
            Self { joined, next_batch: None }
        }

        /// Creates a `Response` with the given room ids, together with a batch token.
        pub fn with_batch_token(joined: Vec<RoomId>, token: String) -> Self {
            Self { joined, next_batch: Some(token) }
        }
    }
}

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.19/client-server-api/#get_matrixclientv1mutual_rooms

    use js_int::UInt;
    use ruma_common::{
        OwnedUserId, RoomId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.19 | stable("uk.half-shot.msc2666.query_mutual_rooms.stable") => "/_matrix/client/v1/mutual_rooms",
        }
    }

    /// Request type for the `mutual_rooms` endpoint.
    #[request]
    pub struct Request {
        /// The user to search mutual rooms for.
        #[ruma_api(query)]
        pub user_id: OwnedUserId,

        /// The `next_batch` returned from a previous response, to get the next batch of
        /// rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub from: Option<String>,
    }

    /// Response type for the `mutual_rooms` endpoint.
    #[response]
    pub struct Response {
        /// The total number of rooms the user is in together with the authenticated user.
        ///
        /// This is unaffected by batching.
        pub count: UInt,

        /// A list of rooms the user is in together with the authenticated user.
        pub joined: Vec<RoomId>,

        /// An opaque string, returned when the server paginates this response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, from: None }
        }

        /// Creates a new `Request` with the given user id, together with a batch token.
        pub fn with_batch_token(user_id: OwnedUserId, token: String) -> Self {
            Self { user_id, from: Some(token) }
        }
    }

    impl Response {
        /// Creates a `Response` with the given count and room IDs.
        pub fn new(count: UInt, joined: Vec<RoomId>) -> Self {
            Self { count, joined, next_batch: None }
        }

        /// Creates a `Response` with the given count and room IDs, together with a batch token.
        pub fn with_batch_token(count: UInt, joined: Vec<RoomId>, token: String) -> Self {
            Self { count, joined, next_batch: Some(token) }
        }
    }
}
