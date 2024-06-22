//! `POST /_matrix/client/*/logout/all`
//!
//! Invalidates all access tokens for a user, so that they can no longer be used for authorization.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3logoutall

    use ruma_common::{
        api::{response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/logout/all",
            1.1 => "/_matrix/client/v3/logout/all",
        }
    };

    /// Request type for the `logout_all` endpoint.
    #[derive(Debug, Clone, Default)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct Request {}

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    #[cfg(feature = "client")]
    impl ruma_common::api::OutgoingRequest for Request {
        type EndpointError = crate::Error;
        type IncomingResponse = Response;

        const METADATA: Metadata = METADATA;

        fn try_into_http_request<T: Default + bytes::BufMut>(
            self,
            base_url: &str,
            access_token: ruma_common::api::SendAccessToken<'_>,
            considering_versions: &'_ [ruma_common::api::MatrixVersion],
        ) -> Result<http::Request<T>, ruma_common::api::error::IntoHttpError> {
            let url = METADATA.make_endpoint_url(considering_versions, base_url, &[], "")?;

            http::Request::builder()
                .method(METADATA.method)
                .uri(url)
                .header(
                    http::header::AUTHORIZATION,
                    format!(
                        "Bearer {}",
                        access_token
                            .get_required_for_endpoint()
                            .ok_or(ruma_common::api::error::IntoHttpError::NeedsAuthentication)?,
                    ),
                )
                .body(T::default())
                .map_err(Into::into)
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
            if request.method() != METADATA.method {
                return Err(ruma_common::api::error::FromHttpRequestError::MethodMismatch {
                    expected: METADATA.method,
                    received: request.method().clone(),
                });
            }

            Ok(Self {})
        }
    }

    /// Response type for the `logout_all` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
