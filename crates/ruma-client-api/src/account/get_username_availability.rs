//! `GET /_matrix/client/*/register/available`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3registeravailable

    use ruma_common::api::ruma_api;

    ruma_api! {
        metadata: {
            description: "Checks to see if a username is available, and valid, for the server.",
            method: GET,
            name: "get_username_availability",
            r0_path: "/_matrix/client/r0/register/available",
            stable_path: "/_matrix/client/v3/register/available",
            rate_limited: true,
            authentication: None,
            added: 1.0,
        }

        request: {
            /// The username to check the availability of.
            #[ruma_api(query)]
            pub username: &'a str,
        }

        response: {
            /// A flag to indicate that the username is available.
            /// This should always be true when the server replies with 200 OK.
            pub available: bool,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given username.
        pub fn new(username: &'a str) -> Self {
            Self { username }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given availability flag.
        pub fn new(available: bool) -> Self {
            Self { available }
        }
    }
}
