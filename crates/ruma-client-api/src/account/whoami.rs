//! `GET /_matrix/client/*/account/whoami`
//!
//! Get information about the owner of a given access token.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3accountwhoami

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedDeviceId, OwnedUserId,
    };

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/account/whoami",
            1.1 => "/_matrix/client/v3/account/whoami",
        }
    };

    /// Request type for the `whoami` endpoint.
    #[request(error = crate::Error)]
    #[derive(Default)]
    pub struct Request {}

    /// Response type for the `whoami` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// The id of the user that owns the access token.
        pub user_id: OwnedUserId,

        /// The device ID associated with the access token, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_id: Option<OwnedDeviceId>,

        /// If `true`, the user is a guest user.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub is_guest: bool,
    }

    impl Request {
        /// Creates an empty `Request`.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Response {
        /// Creates a new `Response` with the given user ID.
        pub fn new(user_id: OwnedUserId, is_guest: bool) -> Self {
            Self { user_id, device_id: None, is_guest }
        }
    }
}
