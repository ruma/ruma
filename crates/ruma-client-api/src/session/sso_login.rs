//! `GET /_matrix/client/*/login/sso/redirect`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3loginssoredirect

    use http::header::{LOCATION, SET_COOKIE};
    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/client/r0/login/sso/redirect",
            1.1 => "/_matrix/client/v3/login/sso/redirect",
        }
    };

    /// Request type for the `sso_login` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// URL to which the homeserver should return the user after completing
        /// authentication with the SSO identity provider.
        #[ruma_api(query)]
        #[serde(rename = "redirectUrl")]
        pub redirect_url: String,
    }

    /// Response type for the `sso_login` endpoint.
    #[response(error = crate::Error, status = FOUND)]
    pub struct Response {
        /// Redirect URL to the SSO identity provider.
        #[ruma_api(header = LOCATION)]
        pub location: String,

        /// Cookie storing state to secure the SSO process.
        #[ruma_api(header = SET_COOKIE)]
        pub cookie: Option<String>,
    }

    impl Request {
        /// Creates a new `Request` with the given redirect URL.
        pub fn new(redirect_url: String) -> Self {
            Self { redirect_url }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given SSO URL.
        pub fn new(location: String) -> Self {
            Self { location, cookie: None }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod tests {
        use ruma_common::api::{MatrixVersion, OutgoingRequest, SendAccessToken};

        use super::Request;

        #[test]
        fn serialize_sso_login_request_uri() {
            let req: http::Request<Vec<u8>> =
                Request { redirect_url: "https://example.com/sso".to_owned() }
                    .try_into_http_request(
                        "https://homeserver.tld",
                        SendAccessToken::None,
                        &[MatrixVersion::V1_1],
                    )
                    .unwrap();

            assert_eq!(
            req.uri().to_string(),
            "https://homeserver.tld/_matrix/client/v3/login/sso/redirect?redirectUrl=https%3A%2F%2Fexample.com%2Fsso"
        );
        }
    }
}
