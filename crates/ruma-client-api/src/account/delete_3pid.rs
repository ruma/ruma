//! POST /_matrix/client/*/account/3pid/delete
//!
//! Delete a 3PID from a user's account on an identity server.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3account3piddelete

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Medium,
    };

    use crate::account::ThirdPartyIdRemovalStatus;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/account/3pid/delete",
            1.1 => "/_matrix/client/v3/account/3pid/delete",
        }
    };

    /// Request type for the `delete_3pid` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// Identity server to delete from.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<String>,

        /// Medium of the 3PID to be removed.
        pub medium: Medium,

        /// Third-party address being removed.
        pub address: String,
    }

    /// Response type for the `delete_3pid` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// Result of unbind operation.
        pub id_server_unbind_result: ThirdPartyIdRemovalStatus,
    }

    impl Request {
        /// Creates a new `Request` with the given medium and address.
        pub fn new(medium: Medium, address: String) -> Self {
            Self { id_server: None, medium, address }
        }
    }
}
