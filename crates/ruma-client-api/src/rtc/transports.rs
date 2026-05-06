//! `GET /_matrix/client/*/rtc/transports`
//!
//! Discover the RTC transports advertised by the homeserver.

pub mod v1 {
    //! `/v1/` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143

    use ruma_common::{
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    use crate::rtc::RtcTransport;

    metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4143/rtc/transports",
        }
    }

    /// Request type for the `transports` endpoint.
    #[request]
    #[derive(Default)]
    pub struct Request {}

    impl Request {
        /// Creates a new empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    /// Response type for the `transports` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {
        /// The RTC transports advertised by the homeserver.
        pub rtc_transports: Vec<RtcTransport>,
    }

    impl Response {
        /// Creates a `Response` with the given RTC transports.
        pub fn new(rtc_transports: Vec<RtcTransport>) -> Self {
            Self { rtc_transports }
        }
    }
}
