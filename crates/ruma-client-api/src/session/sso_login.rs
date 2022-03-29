//! `GET /_matrix/client/*/login/sso/redirect`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3loginssoredirect

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "",
            method: GET,
            name: "sso_login",
            r0_path: "/_matrix/client/r0/login/sso/redirect",
            stable_path: "/_matrix/client/v3/login/sso/redirect",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        request: {
            /// URL to which the homeserver should return the user after completing
            /// authentication with the SSO identity provider.
            #[ruma_api(query)]
            #[serde(rename = "redirectUrl")]
            pub redirect_url: &'a str,
        }

        response: {
            /// Redirect URL to the SSO identity provider.
            #[ruma_api(header = LOCATION)]
            pub location: String,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given redirect URL.
        pub fn new(redirect_url: &'a str) -> Self {
            Self { redirect_url }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given SSO URL.
        pub fn new(location: String) -> Self {
            Self { location }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod tests {
        use ruma_common::api::{MatrixVersion, OutgoingRequest, SendAccessToken};

        use super::Request;

        #[test]
        fn serialize_sso_login_request_uri() {
            let req: http::Request<Vec<u8>> = Request { redirect_url: "https://example.com/sso" }
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
