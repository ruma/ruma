//! `GET /_matrix/client/*/login/sso/redirect/{idpId}`
//!
//! Get the SSO login identity provider url.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3loginssoredirectidpid

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
            unstable => "/_matrix/client/unstable/org.matrix.msc2858/login/sso/redirect/:idp_id",
            1.1 => "/_matrix/client/v3/login/sso/redirect/:idp_id",
        }
    };

    /// Request type for the `sso_login_with_provider` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The ID of the provider to use for SSO login.
        #[ruma_api(path)]
        pub idp_id: String,

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

    /// Response type for the `sso_login_with_provider` endpoint.
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
        /// Creates a new `Request` with the given identity provider ID and redirect URL.
        pub fn new(idp_id: String, redirect_url: String) -> Self {
            Self {
                idp_id,
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
            MatrixVersion, OutgoingRequest as _, SendAccessToken, SupportedVersions,
        };

        use super::Request;

        #[test]
        fn serialize_sso_login_with_provider_request_uri() {
            let supported =
                SupportedVersions { versions: [MatrixVersion::V1_1].into(), features: Vec::new() };
            let req = Request::new("provider".to_owned(), "https://example.com/sso".to_owned())
                .try_into_http_request::<Vec<u8>>(
                    "https://homeserver.tld",
                    SendAccessToken::None,
                    &supported,
                )
                .unwrap();

            assert_eq!(
            req.uri().to_string(),
            "https://homeserver.tld/_matrix/client/v3/login/sso/redirect/provider?redirectUrl=https%3A%2F%2Fexample.com%2Fsso"
        );
        }
    }
}
