//! `GET /_matrix/client/*/auth/{auth_type}/fallback/web?session={session_id}`
//!
//! Get UIAA fallback web page.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#fallback

    use http::header::LOCATION;
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/auth/:auth_type/fallback/web",
            1.1 => "/_matrix/client/v3/auth/:auth_type/fallback/web",
        }
    };

    /// Request type for the `authorize_fallback` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The type name ("m.login.dummy", etc.) of the uiaa stage to get a fallback page for.
        #[ruma_api(path)]
        pub auth_type: &'a str,

        /// The ID of the session given by the homeserver.
        #[ruma_api(query)]
        pub session: &'a str,
    }

    /// Response type for the `authorize_fallback` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// Optional URI to redirect to.
        #[ruma_api(header = LOCATION)]
        pub redirect_url: Option<String>,

        /// HTML to return to client.
        #[ruma_api(raw_body)]
        pub body: Vec<u8>,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given auth type and session ID.
        pub fn new(auth_type: &'a str, session: &'a str) -> Self {
            Self { auth_type, session }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given HTML body.
        pub fn new(body: Vec<u8>) -> Self {
            Self { redirect_url: None, body }
        }

        /// Creates a new `Response` with the given redirect URL and an empty body.
        pub fn redirect(url: String) -> Self {
            Self { redirect_url: Some(url), body: Vec::new() }
        }
    }
}
