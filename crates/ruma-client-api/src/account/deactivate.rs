//! `POST /_matrix/client/*/account/deactivate`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3accountdeactivate

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
    };

    use crate::{
        account::ThirdPartyIdRemovalStatus,
        uiaa::{AuthData, IncomingAuthData, UiaaResponse},
    };

    const METADATA: Metadata = metadata! {
        description: "Deactivate the current user's account.",
        method: POST,
        name: "deactivate",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/account/deactivate",
            1.1 => "/_matrix/client/v3/account/deactivate",
        }
    };

    #[request(error = UiaaResponse)]
    #[derive(Default)]
    pub struct Request<'a> {
        /// Additional authentication information for the user-interactive authentication API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth: Option<AuthData<'a>>,

        /// Identity server from which to unbind the user's third party
        /// identifier.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<&'a str>,
    }

    #[response(error = UiaaResponse)]
    pub struct Response {
        /// Result of unbind operation.
        pub id_server_unbind_result: ThirdPartyIdRemovalStatus,
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
