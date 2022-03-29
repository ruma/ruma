//! `GET /_matrix/client/*/auth/{auth_type}/fallback/web?session={session_id}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#fallback

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Get UIAA fallback web page.",
            method: GET,
            name: "authorize_fallback",
            r0_path: "/_matrix/client/r0/auth/:auth_type/fallback/web",
            stable_path: "/_matrix/client/v3/auth/:auth_type/fallback/web",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        request: {
            /// The type name ("m.login.dummy", etc.) of the uiaa stage to get a fallback page for.
            #[ruma_api(path)]
            pub auth_type: String,

            /// The ID of the session given by the homeserver.
            #[ruma_api(query)]
            pub session: String,
        }

        #[derive(Default)]
        response: {
            /// Optional URI to redirect to.
            #[ruma_api(header = LOCATION)]
            pub redirect_url: Option<String>,

            /// HTML to return to client.
            #[ruma_api(raw_body)]
            pub body: Vec<u8>,
        }

        error: crate::Error
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
            Self { redirect_url: None, body }
        }

        /// Creates a new `Response` with the given redirect URL and an empty body.
        pub fn redirect(url: String) -> Self {
            Self { redirect_url: Some(url), body: Vec::new() }
        }
    }
}
