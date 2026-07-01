//! `GET /_matrix/federation/*/query/presence_recipients`
pub mod msc4495 {
    //! [`/unstable/org.continuwuity.presence_v2.msc4495/`][MSC4495]
    //!
    //! [MSC4495]: https://github.com/matrix-org/matrix-spec-proposals/pull/4495

    use js_int::UInt;
    use ruma_common::{
        OwnedUserId,
        api::{request, response},
        metadata,
    };

    use crate::authentication::ServerSignatures;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/unstable/org.continuwuity.presence_v2.msc4495/query/presence_recipients",
    }

    /// Request type for the `get_presence_recipients` endpoint.
    #[request]
    pub struct Request {
        /// The user ID to query. Must be local to the queried server.
        #[ruma_api(query)]
        pub user_id: OwnedUserId,
    }

    impl Request {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id }
        }
    }

    /// Response type for the `get_presence_recipients` endpoint.
    #[response]
    pub struct Response {
        /// A unique identifier for the user's current recipient user set.
        pub stream_id: UInt,

        /// An array of local recipients the user intends to push presence to.
        pub recipients: Vec<OwnedUserId>,
    }

    impl Response {
        /// Creates a new `Response` with the given stream ID and recipients.
        pub fn new(stream_id: UInt, recipients: Vec<OwnedUserId>) -> Self {
            Self { stream_id, recipients }
        }
    }
}
