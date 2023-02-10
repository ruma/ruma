//! `POST /_matrix/client/*/account/3pid/bind`
//!
//! Bind a 3PID to a user's account on an identity server

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#post_matrixclientv3account3pidbind

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedClientSecret, OwnedSessionId,
    };

    use crate::account::IdentityServerInfo;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/account/3pid/bind",
            1.1 => "/_matrix/client/v3/account/3pid/bind",
        }
    };

    /// Request type for the `bind_3pid` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// Client-generated secret string used to protect this session.
        pub client_secret: OwnedClientSecret,

        /// The ID server to send the onward request to as a hostname with an
        /// appended colon and port number if the port is not the default.
        #[serde(flatten)]
        pub identity_server_info: IdentityServerInfo,

        /// The session identifier given by the identity server.
        pub sid: OwnedSessionId,
    }

    /// Response type for the `bind_3pid` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given client secret, identity server information and
        /// session identifier.
        pub fn new(
            client_secret: OwnedClientSecret,
            identity_server_info: IdentityServerInfo,
            sid: OwnedSessionId,
        ) -> Self {
            Self { client_secret, identity_server_info, sid }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
