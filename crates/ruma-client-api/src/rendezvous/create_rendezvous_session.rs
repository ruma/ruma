//! `POST /_matrix/client/*/rendezvous/`
//!
//! Create a rendezvous session.

pub mod unstable {
    //! `msc4108` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4108

    use http::{
        header::{CONTENT_LENGTH, CONTENT_TYPE, ETAG, EXPIRES, LAST_MODIFIED},
        HeaderName,
    };
    #[cfg(feature = "client")]
    use ruma_common::api::error::FromHttpResponseError;
    use ruma_common::{
        api::{error::HeaderDeserializationError, Metadata},
        metadata,
    };
    use serde::{Deserialize, Serialize};
    use url::Url;
    use web_time::SystemTime;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: None,
        history: {
            unstable => "/_matrix/client/unstable/org.matrix.msc4108/rendezvous",
        }
    };

    /// Request type for the `POST` `rendezvous` endpoint.
    #[derive(Debug, Default, Clone)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Request {
        /// Any data up to maximum size allowed by the server.
        pub content: String,
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;
        const METADATA: Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            _: ruma_common::api::SendAccessToken<'_>,
            considering_versions: &'_ [ruma_common::api::MatrixVersion],
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            let url = METADATA.make_endpoint_url(considering_versions, base_url, &[], "")?;
            let body = self.content.as_bytes();
            let content_length = body.len();

            Ok(http::Request::builder()
                .method(METADATA.method)
                .uri(url)
                .header(CONTENT_TYPE, "text/plain")
                .header(CONTENT_LENGTH, content_length)
                .body(ruma_common::serde::slice_to_buf(body))?)
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::IncomingRequest for Request {
        type EndpointError = crate::Error;
        type OutgoingResponse = Response;
        const METADATA: Metadata = METADATA;

        fn try_from_http_request<B, S>(
            request: http::Request<B>,
            _path_args: &[S],
        ) -> Result<Self, ruma_common::api::error::FromHttpRequestError>
        where
            B: AsRef<[u8]>,
            S: AsRef<str>,
        {
            const EXPECTED_CONTENT_TYPE: &str = "text/plain";

            use ruma_common::api::error::DeserializationError;

            let content_type = request
                .headers()
                .get(CONTENT_TYPE)
                .ok_or(HeaderDeserializationError::MissingHeader(CONTENT_TYPE.to_string()))?;

            let content_type = content_type.to_str()?;

            if content_type != EXPECTED_CONTENT_TYPE {
                Err(HeaderDeserializationError::InvalidHeaderValue {
                    header: CONTENT_TYPE.to_string(),
                    expected: EXPECTED_CONTENT_TYPE.to_owned(),
                    unexpected: content_type.to_owned(),
                }
                .into())
            } else {
                let body = request.into_body().as_ref().to_vec();
                let content = String::from_utf8(body)
                    .map_err(|e| DeserializationError::Utf8(e.utf8_error()))?;

                Ok(Self { content })
            }
        }
    }

    impl Request {
        /// Creates a new `Request` with the given content.
        pub fn new(content: String) -> Self {
            Self { content }
        }
    }

    /// Response type for the `POST` `rendezvous` endpoint.
    #[derive(Debug, Clone)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Response {
        /// The absolute URL of the rendezvous session.
        pub url: Url,

        /// ETag for the current payload at the rendezvous session as
        /// per [RFC7232](https://httpwg.org/specs/rfc7232.html#header.etag).
        pub etag: String,

        /// The expiry time of the rendezvous as per
        /// [RFC7234](https://httpwg.org/specs/rfc7234.html#header.expires).
        pub expires: SystemTime,

        /// The last modified date of the payload as
        /// per [RFC7232](https://httpwg.org/specs/rfc7232.html#header.last-modified)
        pub last_modified: SystemTime,
    }

    #[derive(Serialize, Deserialize)]
    struct ResponseBody {
        url: Url,
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::IncomingResponse for Response {
        type EndpointError = crate::Error;

        fn try_from_http_response<T: AsRef<[u8]>>(
            response: http::Response<T>,
        ) -> Result<Self, FromHttpResponseError<Self::EndpointError>> {
            use ruma_common::api::EndpointError;

            if response.status().as_u16() >= 400 {
                return Err(FromHttpResponseError::Server(
                    Self::EndpointError::from_http_response(response),
                ));
            }

            let get_date = |header: HeaderName| -> Result<SystemTime, FromHttpResponseError<Self::EndpointError>> {
                let date = response
                    .headers()
                    .get(&header)
                    .ok_or_else(|| HeaderDeserializationError::MissingHeader(header.to_string()))?;

                let date = crate::http_headers::http_date_to_system_time(date)?;

                Ok(date)
            };

            let etag = response
                .headers()
                .get(ETAG)
                .ok_or(HeaderDeserializationError::MissingHeader(ETAG.to_string()))?
                .to_str()?
                .to_owned();
            let expires = get_date(EXPIRES)?;
            let last_modified = get_date(LAST_MODIFIED)?;

            let body = response.into_body();
            let body: ResponseBody = serde_json::from_slice(body.as_ref())?;

            Ok(Self { url: body.url, etag, expires, last_modified })
        }
    }

    #[cfg(feature = "server")]
    impl ruma_common::api::OutgoingResponse for Response {
        fn try_into_http_response<T: Default + bytes::BufMut>(
            self,
        ) -> Result<http::Response<T>, ruma_common::api::error::IntoHttpError> {
            use http::header::{CACHE_CONTROL, PRAGMA};

            let body = ResponseBody { url: self.url.clone() };
            let body = serde_json::to_vec(&body)?;
            let body = ruma_common::serde::slice_to_buf(&body);

            let expires = crate::http_headers::system_time_to_http_date(&self.expires)?;
            let last_modified = crate::http_headers::system_time_to_http_date(&self.last_modified)?;

            Ok(http::Response::builder()
                .status(http::StatusCode::OK)
                .header(CONTENT_TYPE, "application/json")
                .header(PRAGMA, "no-cache")
                .header(CACHE_CONTROL, "no-store")
                .header(ETAG, self.etag)
                .header(EXPIRES, expires)
                .header(LAST_MODIFIED, last_modified)
                .body(body)?)
        }
    }
}
