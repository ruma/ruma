//! `POST /_matrix/client/*/account/3pid/unbind`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3account3pidunbind

    use ruma_common::{api::ruma_api, thirdparty::Medium};

    use crate::account::ThirdPartyIdRemovalStatus;

    ruma_api! {
        metadata: {
            description: "Unbind a 3PID from a user's account on an identity server.",
            method: POST,
            name: "unbind_3pid",
            r0_path: "/_matrix/client/r0/account/3pid/unbind",
            stable_path: "/_matrix/client/v3/account/3pid/unbind",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// Identity server to unbind from.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub id_server: Option<&'a str>,

            /// Medium of the 3PID to be removed.
            pub medium: Medium,

            /// Third-party address being removed.
            pub address: &'a str,
        }

        response: {
            /// Result of unbind operation.
            pub id_server_unbind_result: ThirdPartyIdRemovalStatus,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given medium and third-party address.
        pub fn new(medium: Medium, address: &'a str) -> Self {
            Self { id_server: None, medium, address }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given unbind result.
        pub fn new(id_server_unbind_result: ThirdPartyIdRemovalStatus) -> Self {
            Self { id_server_unbind_result }
        }
    }
}
