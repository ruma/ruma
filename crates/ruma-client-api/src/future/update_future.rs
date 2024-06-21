//! `POST /_matrix/client/*/futures/{token}`
//!
//! Send a future token to update/cancel/send the associated future event.

pub mod unstable {
    //! `msc3814` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: None,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4140/future/:token",
        }
    };

    /// Request type for the [`update_future`](crate::future::update_future) endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The token.
        #[ruma_api(path)]
        pub token: String,
    }

    impl Request {
        /// Creates a new `Request` to update a future. This is an unauthenticated request and only
        /// requires the future token.
        pub fn new(token: String) -> serde_json::Result<Self> {
            Ok(Self { token })
        }
    }

    /// Response type for the [`update_future`](crate::future::update_future) endpoint.
    #[response(error = crate::Error)]
    pub struct Response {}
    impl Response {
        /// Creates a new response for the [`update_future`](crate::future::update_future) endpoint.
        pub fn new() -> Self {
            Response {}
        }
    }
}
