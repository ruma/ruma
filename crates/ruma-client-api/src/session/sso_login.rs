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

    #[cfg(feature = "unstable-msc3824")]
    use crate::session::SsoRedirectOidcAction;

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

        /// The purpose for using the SSO redirect URL for OIDC-aware compatibility.
        ///
        /// This field uses the unstable prefix defined in [MSC3824].
        ///
        /// [MSC3824]: https://github.com/matrix-org/matrix-spec-proposals/pull/3824
        #[cfg(feature = "unstable-msc3824")]
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none", rename = "org.matrix.msc3824.action")]
        pub action: Option<SsoRedirectOidcAction>,
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
            Self {
                redirect_url,
                #[cfg(feature = "unstable-msc3824")]
                action: None,
            }
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
        use ruma_common::api::{
            MatrixVersion, OutgoingRequest, SendAccessToken, SupportedVersions,
        };

        use super::Request;

        #[test]
        fn serialize_sso_login_request_uri() {
            let supported =
                SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Vec::new() };
            let req: http::Request<Vec<u8>> = Request::new("https://example.com/sso".to_owned())
                .try_into_http_request("https://homeserver.tld", SendAccessToken::None, &supported)
                .unwrap();

            assert_eq!(
            req.uri().to_string(),
            "https://homeserver.tld/_matrix/client/v3/login/sso/redirect?redirectUrl=https%3A%2F%2Fexample.com%2Fsso"
        );
        }
    }
}
