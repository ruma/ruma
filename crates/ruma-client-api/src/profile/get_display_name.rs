//! `GET /_matrix/client/*/profile/{userId}/displayname`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3profileuseriddisplayname

    use ruma_common::{api::ruma_api, UserId};

    ruma_api! {
        metadata: {
            description: "Get the display name of a user.",
            method: GET,
            name: "get_display_name",
            r0_path: "/_matrix/client/r0/profile/:user_id/displayname",
            stable_path: "/_matrix/client/v3/profile/:user_id/displayname",
            rate_limited: false,
            authentication: None,
            added: 1.0,
        }

        request: {
            /// The user whose display name will be retrieved.
            #[ruma_api(path)]
            pub user_id: &'a UserId,
        }

        #[derive(Default)]
        response: {
            /// The user's display name, if set.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub displayname: Option<String>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID.
        pub fn new(user_id: &'a UserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given display name.
        pub fn new(displayname: Option<String>) -> Self {
            Self { displayname }
        }
    }
}
