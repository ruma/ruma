//! `POST /_matrix/client/*/account/deactivate`
//!
//! Deactivate the current user's account.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3accountdeactivate

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::{
        account::ThirdPartyIdRemovalStatus,
        uiaa::{AuthData, UiaaResponse},
    };

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/account/deactivate",
            1.1 => "/_matrix/client/v3/account/deactivate",
        }
    };

    /// Request type for the `deactivate` endpoint.
    #[request(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Request {
        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData>,

        /// Identity server from which to unbind the user's third party
        /// identifier.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<String>,

        /// Whether the user would like their content to be erased as much as possible from the
        /// server.
        ///
        /// Defaults to `false`.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub erase: bool,
    }

    /// Response type for the `deactivate` endpoint.
    #[response(error = UiaaResponse)]
    pub struct Response {
        /// Result of unbind operation.
        pub id_server_unbind_result: ThirdPartyIdRemovalStatus,
    }

    impl Request {
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
