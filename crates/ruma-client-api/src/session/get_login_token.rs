//! `GET /_matrix/client/*/login/get_token`
//!
//! Generate a single-use, time-limited, `m.login.token` token.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv1loginget_token

    use std::time::Duration;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::uiaa::{AuthData, UiaaResponse};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc3882") => "/_matrix/client/unstable/org.matrix.msc3882/login/get_token",
            1.7 => "/_matrix/client/v1/login/get_token",
        }
    };

    /// Request type for the `login` endpoint.
    #[request(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Request {
        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData>,
    }

    /// Response type for the `login` endpoint.
    #[response(error = UiaaResponse)]
    pub struct Response {
        /// The time remaining in milliseconds until the homeserver will no longer accept the
        /// token.
        ///
        /// 2 minutes is recommended as a default.
        #[serde(with = "ruma_common::serde::duration::ms", rename = "expires_in_ms")]
        pub expires_in: Duration,

        /// The login token for the `m.login.token` login flow.
        pub login_token: String,
    }

    impl Request {
        /// Creates a new empty `Request`.
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Response {
        /// Creates a new `Response` with the given expiration duration and login token.
        pub fn new(expires_in: Duration, login_token: String) -> Self {
            Self { expires_in, login_token }
        }

        /// Creates a new `Response` with the default expiration duration and the given login token.
        pub fn with_default_expiration_duration(login_token: String) -> Self {
            Self::new(Self::default_expiration_duration(), login_token)
        }

        fn default_expiration_duration() -> Duration {
            // 2 minutes.
            Duration::from_secs(2 * 60)
        }
    }
}
