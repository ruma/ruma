//! POST /_matrix/client/*/account/3pid/delete

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3account3piddelete

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Medium,
    };

    use crate::account::ThirdPartyIdRemovalStatus;

    const METADATA: Metadata = metadata! {
        description: "Delete a 3PID from a user's account on an identity server.",
        method: POST,
        name: "delete_3pid",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/account/3pid/delete",
            1.1 => "/_matrix/client/v3/account/3pid/delete",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// Identity server to delete from.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub id_server: Option<&'a str>,

        /// Medium of the 3PID to be removed.
        pub medium: Medium,

        /// Third-party address being removed.
        pub address: &'a str,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// Result of unbind operation.
        pub id_server_unbind_result: ThirdPartyIdRemovalStatus,
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given medium and address.
        pub fn new(medium: Medium, address: &'a str) -> Self {
            Self { id_server: None, medium, address }
        }
    }
}
