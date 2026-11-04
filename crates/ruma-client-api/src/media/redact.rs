//! `POST /_matrix/client/*/media/redact/{server_name}/{media_id}`
//!
//! Redact a file in the content repository.

pub mod unstable {
    //! `/unstable/uk.timedout.msc4322/` ([MSC4322])
    //!
    //! [MSC4322]: https://github.com/matrix-org/matrix-spec-proposals/pull/4322

    use ruma_common::{
        IdParseError, MxcUri, OwnedServerName,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("uk.timedout.msc4322") => "/_matrix/client/unstable/uk.timedout.msc4322/media/redact/{server_name}/{media_id}",
        }
    }

    /// Request type for the `redact` endpoint.
    #[request]
    pub struct Request {
        /// The server name from the mxc:// URI (the authoritory component).
        #[ruma_api(path)]
        pub server_name: OwnedServerName,

        /// The media ID from the mxc:// URI (the path component).
        #[ruma_api(path)]
        pub media_id: String,

        /// The reason this media is being redacted.
        ///
        /// Servers may surface this string in their auditing logs.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given media ID and server name.
        pub fn new(media_id: String, server_name: OwnedServerName) -> Self {
            Self { media_id, server_name, reason: None }
        }

        /// Creates a new `Request` with the given url.
        pub fn from_url(url: &MxcUri) -> Result<Self, IdParseError> {
            let (server_name, media_id) = url.parts()?;

            Ok(Self::new(media_id.to_owned(), server_name.to_owned()))
        }
    }

    /// Response type for the `redact` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Response {
        /// Creates a new `Response`.
        pub fn new() -> Self {
            Self::default()
        }
    }
}
