//! `POST /_matrix/client/*/account/deactivate`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3accountdeactivate

    use ruma_common::api::ruma_api;

    use crate::{
        account::ThirdPartyIdRemovalStatus,
        uiaa::{AuthData, IncomingAuthData, UiaaResponse},
    };

    ruma_api! {
        metadata: {
            description: "Deactivate the current user's account.",
            method: POST,
            name: "deactivate",
            r0_path: "/_matrix/client/r0/account/deactivate",
            stable_path: "/_matrix/client/v3/account/deactivate",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        #[derive(Default)]
        request: {
            /// Additional authentication information for the user-interactive authentication API.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub auth: Option<AuthData<'a>>,

            /// Identity server from which to unbind the user's third party
            /// identifier.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub id_server: Option<&'a str>,
        }

        response: {
            /// Result of unbind operation.
            pub id_server_unbind_result: ThirdPartyIdRemovalStatus,
        }

        error: UiaaResponse
    }

    impl Request<'_> {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    impl Response {
        /// Creates a new `Response` with the given unbind result.
        pub fn new(id_server_unbind_result: ThirdPartyIdRemovalStatus) -> Self {
            Self { id_server_unbind_result }
        }
    }
}
