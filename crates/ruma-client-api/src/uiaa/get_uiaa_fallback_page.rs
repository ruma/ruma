//! `GET /_matrix/client/*/auth/{auth_type}/fallback/web?session={session_id}`
//!
//! Get UIAA fallback web page.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#fallback

    use ruma_common::{
        api::{request, Metadata},
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
    pub struct Request {
        /// The type name ("m.login.dummy", etc.) of the uiaa stage to get a fallback page for.
        #[ruma_api(path)]
        pub auth_type: String,

        /// The ID of the session given by the homeserver.
        #[ruma_api(query)]
        pub session: String,
    }

    impl Request {
        /// Creates a new `Request` with the given auth type and session ID.
        pub fn new(auth_type: String, session: String) -> Self {
            Self { auth_type, session }
        }
    }

    /// Response type for the `authorize_fallback` endpoint.
    #[derive(Debug, Clone)]
    #[allow(clippy::exhaustive_enums)]
    pub enum Response {
        /// The response is a redirect.
        Redirect(Redirect),

        /// The response is an HTML page.
        Html(HtmlPage),
    }

    /// The data of a redirect.
    #[derive(Debug, Clone)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Redirect {
        /// The URL to redirect the user to.
        pub url: String,
    }

    /// The data of a HTML page.
    #[derive(Debug, Clone)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct HtmlPage {
        /// The body of the HTML page.
        pub body: Vec<u8>,
    }

    impl Response {
        /// Creates a new HTML `Response` with the given HTML body.
        pub fn html(body: Vec<u8>) -> Self {
            Self::Html(HtmlPage { body })
        }

        /// Creates a new HTML `Response` with the given redirect URL.
        pub fn redirect(url: String) -> Self {
            Self::Redirect(Redirect { url })
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::OutgoingResponse for Response {
        fn try_into_http_response<T: Default + bytes::BufMut>(
            self,
        ) -> Result<http::Response<T>, ruma_common::api::error::IntoHttpError> {
            match self {
                Response::Redirect(Redirect { url }) => Ok(http::Response::builder()
                    .status(http::StatusCode::FOUND)
                    .header(http::header::LOCATION, url)
                    .body(T::default())?),
                Response::Html(HtmlPage { body }) => Ok(http::Response::builder()
                    .status(http::StatusCode::OK)
                    .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(ruma_common::serde::slice_to_buf(&body))?),
            }
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::IncomingResponse for Response {
        type EndpointError = crate::Error;

        fn try_from_http_response<T: AsRef<[u8]>>(
            response: http::Response<T>,
        ) -> Result<Self, ruma_common::api::error::FromHttpResponseError<Self::EndpointError>>
        {
            use ruma_common::api::{
                error::{DeserializationError, FromHttpResponseError, HeaderDeserializationError},
                EndpointError,
            };

            if response.status().as_u16() >= 400 {
                return Err(FromHttpResponseError::Server(
                    Self::EndpointError::from_http_response(response),
                ));
            }

            if response.status() == http::StatusCode::FOUND {
                let Some(location) = response.headers().get(http::header::LOCATION) else {
                    return Err(DeserializationError::Header(
                        HeaderDeserializationError::MissingHeader(
                            http::header::LOCATION.to_string(),
                        ),
                    )
                    .into());
                };

                let url = location.to_str()?;
                return Ok(Self::Redirect(Redirect { url: url.to_owned() }));
            }

            let body = response.into_body().as_ref().to_owned();
            Ok(Self::Html(HtmlPage { body }))
        }
    }

    #[cfg(all(test, any(feature = "client", feature = "server")))]
    mod tests {
        use assert_matches2::assert_matches;
        use http::header::{CONTENT_TYPE, LOCATION};
        #[cfg(feature = "client")]
        use ruma_common::api::IncomingResponse;
        #[cfg(feature = "server")]
        use ruma_common::api::OutgoingResponse;

        use super::Response;

        #[cfg(feature = "client")]
        #[test]
        fn incoming_redirect() {
            use super::Redirect;

            let http_response = http::Response::builder()
                .status(http::StatusCode::FOUND)
                .header(LOCATION, "http://localhost/redirect")
                .body(Vec::<u8>::new())
                .unwrap();

            let response = Response::try_from_http_response(http_response).unwrap();
            assert_matches!(response, Response::Redirect(Redirect { url }));
            assert_eq!(url, "http://localhost/redirect");
        }

        #[cfg(feature = "client")]
        #[test]
        fn incoming_html() {
            use super::HtmlPage;

            let http_response = http::Response::builder()
                .status(http::StatusCode::OK)
                .header(CONTENT_TYPE, "text/html; charset=utf-8")
                .body(b"<h1>My Page</h1>")
                .unwrap();

            let response = Response::try_from_http_response(http_response).unwrap();
            assert_matches!(response, Response::Html(HtmlPage { body }));
            assert_eq!(body, b"<h1>My Page</h1>");
        }

        #[cfg(feature = "server")]
        #[test]
        fn outgoing_redirect() {
            let response = Response::redirect("http://localhost/redirect".to_owned());

            let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

            assert_eq!(http_response.status(), http::StatusCode::FOUND);
            assert_eq!(
                http_response.headers().get(LOCATION).unwrap().to_str().unwrap(),
                "http://localhost/redirect"
            );
            assert!(http_response.into_body().is_empty());
        }

        #[cfg(feature = "server")]
        #[test]
        fn outgoing_html() {
            let response = Response::html(b"<h1>My Page</h1>".to_vec());

            let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();

            assert_eq!(http_response.status(), http::StatusCode::OK);
            assert_eq!(
                http_response.headers().get(CONTENT_TYPE).unwrap().to_str().unwrap(),
                "text/html; charset=utf-8"
            );
            assert_eq!(http_response.into_body(), b"<h1>My Page</h1>");
        }
    }
}
