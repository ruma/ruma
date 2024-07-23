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
    use serde::{Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4140/update_future",
        }
    };

    /// The possible update actions we can do for updating a future.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "lowercase")]
    pub enum UpdateAction {
        /// Restart the Future event timeout. (heartbeat ping)
        Restart,
        /// Send the Future event immediately independent of the timeout state. (deletes all
        /// timers)
        Send,
        /// Delete the Future event and never send it. (deletes all timers)
        Cancel,
    }
    /// Request type for the [`update_future`](crate::future::update_future) endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The future if that we want to update.
        pub future_id: String,
        /// Which kind of update we want to request for the Future event.
        pub action: UpdateAction,
    }

    impl Request {
        /// Creates a new `Request` to update a future. This is an unauthenticated request and only
        /// requires the future token.
        pub fn new(future_id: String, action: UpdateAction) -> serde_json::Result<Self> {
            Ok(Self { future_id, action })
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

    #[cfg(all(test, feature = "client"))]
    mod tests {
        use ruma_common::api::{MatrixVersion, OutgoingRequest, SendAccessToken};
        use serde_json::{json, Value as JsonValue};

        use super::{Request, UpdateAction};
        #[test]
        fn serialize_update_future_request() {
            let request: http::Request<Vec<u8>> =
                Request::new("1234".to_owned(), UpdateAction::Cancel)
                    .unwrap()
                    .try_into_http_request(
                        "https://homeserver.tld",
                        SendAccessToken::IfRequired("auth_tok"),
                        &[MatrixVersion::V1_1],
                    )
                    .unwrap();

            let (parts, body) = request.into_parts();

            assert_eq!(
                "https://homeserver.tld/_matrix/client/unstable/org.matrix.msc4140/update_future",
                parts.uri.to_string()
            );
            assert_eq!("POST", parts.method.to_string());
            assert_eq!(
                json!({"action": "cancel", "future_id":"1234"}),
                serde_json::from_str::<JsonValue>(std::str::from_utf8(&body).unwrap()).unwrap()
            );
        }
    }
}
