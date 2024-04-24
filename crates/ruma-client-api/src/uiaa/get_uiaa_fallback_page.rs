//! `GET /_matrix/client/*/auth/{auth_type}/fallback/web?session={session_id}`
//!
//! Get UIAA fallback web page.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#fallback

    use http::header::{CONTENT_TYPE, LOCATION};
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
    const HTML: &str = "text/html; charset=utf-8";

    /// Request type for the `authorize_fallback` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The type name ("m.login.dummy", etc.) of the uiaa stage to get a fallback page for.
        #[ruma_api(path)]
        pub auth_type: String,

        /// The ID of the session given by the homeserver.
        #[ruma_api(query)]
        pub session: String,
    }

    /// Response type for the `authorize_fallback` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {
        /// Content type of the body.
        #[ruma_api(header = CONTENT_TYPE)]
        pub content_type: String,

        /// Optional URI to redirect to.
        #[ruma_api(header = LOCATION)]
        pub redirect_url: Option<String>,

        /// HTML to return to client.
        #[ruma_api(raw_body)]
        pub body: Vec<u8>,
    }

    impl Request {
        /// Creates a new `Request` with the given auth type and session ID.
        pub fn new(auth_type: String, session: String) -> Self {
            Self { auth_type, session }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given HTML body.
        pub fn new(body: Vec<u8>) -> Self {
            Self { content_type: HTML.to_owned(), redirect_url: None, body }
        }

        /// Creates a new `Response` with the given redirect URL and an empty body.
        pub fn redirect(url: String) -> Self {
            Self { content_type: HTML.to_owned(), redirect_url: Some(url), body: Vec::new() }
        }
    }
}
