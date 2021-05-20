//! [GET /_matrix/client/r0/auth/{auth_type}/fallback/web?session={session_id}](https://matrix.org/docs/spec/client_server/r0.6.1#fallback)

use ruma_api::ruma_api;

ruma_api! {
    metadata: {
        description: "Get UIAA fallback web page.",
        method: GET,
        name: "uiaa_fallback",
        path: "/_matrix/client/r0/auth/:auth_type/fallback/web",
        rate_limited: false,
        authentication: None,
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
        pub redirect_uri: Option<String>,

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
    /// Creates an new `Response` with redirect uri and raw HTML body.
    pub fn new(redirect_uri: Option<String>, body: Vec<u8>) -> Self {
        Self { redirect_uri, body }
    }
}
