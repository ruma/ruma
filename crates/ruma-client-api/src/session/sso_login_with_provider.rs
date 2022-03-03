//! `GET /_matrix/client/*/login/sso/redirect/{idpId}`
//!
//! Get the SSO login identity provider url.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3loginssoredirectidpid

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Get the SSO login identity provider url.",
            method: GET,
            name: "sso_login_with_provider",
            unstable_path: "/_matrix/client/unstable/org.matrix.msc2858/login/sso/redirect/:idp_id",
            stable_path: "/_matrix/client/v3/login/sso/redirect/:idp_id",
            rate_limited: false,
            authentication: None,
            added: 1.1,
        }

        request: {
            /// The ID of the provider to use for SSO login.
            #[ruma_api(path)]
            pub idp_id: &'a str,

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
        /// Creates a new `Request` with the given identity provider ID and redirect URL.
        pub fn new(idp_id: &'a str, redirect_url: &'a str) -> Self {
            Self { idp_id, redirect_url }
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
        use ruma_common::api::{MatrixVersion, OutgoingRequest as _, SendAccessToken};

        use super::Request;

        #[test]
        fn serialize_sso_login_with_provider_request_uri() {
            let req = Request { idp_id: "provider", redirect_url: "https://example.com/sso" }
                .try_into_http_request::<Vec<u8>>(
                    "https://homeserver.tld",
                    SendAccessToken::None,
                    &[MatrixVersion::V1_1],
                )
                .unwrap();

            assert_eq!(
            req.uri().to_string(),
            "https://homeserver.tld/_matrix/client/v3/login/sso/redirect/provider?redirectUrl=https%3A%2F%2Fexample.com%2Fsso"
        );
        }
    }
}
