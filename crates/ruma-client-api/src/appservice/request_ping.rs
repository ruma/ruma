//! `POST /_matrix/client/*/appservice/{appserviceId}/ping}`
//!
//! Ask the homeserver to ping the application service to ensure the connection works.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#post_matrixclientv1appserviceappserviceidping

    use std::time::Duration;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedTransactionId,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/fi.mau.msc2659/appservice/{appservice_id}/ping",
            1.7 => "/_matrix/client/v1/appservice/{appservice_id}/ping",
        }
    };

    /// Request type for the `request_ping` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The appservice ID of the appservice to ping.
        ///
        /// This must be the same as the appservice whose `as_token` is being used to authenticate
        /// the request.
        #[ruma_api(path)]
        pub appservice_id: String,

        /// Transaction ID that is passed through to the `POST /_matrix/app/v1/ping` call.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub transaction_id: Option<OwnedTransactionId>,
    }

    /// Response type for the `request_ping` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The duration in milliseconds that the `POST /_matrix/app/v1/ping` request took from the
        /// homeserver's point of view.
        #[serde(with = "ruma_common::serde::duration::ms", rename = "duration_ms")]
        pub duration: Duration,
    }

    impl Request {
        /// Creates a new `Request` with the given appservice ID.
        pub fn new(appservice_id: String) -> Self {
            Self { appservice_id, transaction_id: None }
        }
    }

    impl Response {
        /// Creates an `Response` with the given duration.
        pub fn new(duration: Duration) -> Self {
            Self { duration }
        }
    }
}
