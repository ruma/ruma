//! `PUT /_matrix/app/*/ping`
//!
//! Endpoint to ping the application service.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#post_matrixappv1ping

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedTransactionId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/app/unstable/fi.mau.msc2659/ping",
            1.7 => "/_matrix/app/v1/ping",
        }
    };

    /// Request type for the `send_ping` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {
        /// A transaction ID for the ping, copied directly from the `POST
        /// /_matrix/client/v1/appservice/{appserviceId}/ping` call.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub transaction_id: Option<OwnedTransactionId>,
    }

    /// Response type for the `send_ping` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new empty `Request`.
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Response {
        /// Creates a new empty `Response`.
        pub fn new() -> Self {
            Self::default()
        }
    }
}
