//! `PUT /_matrix/client/*/profile/{userId}/displayname`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#put_matrixclientv3profileuseriddisplayname

    use ruma_common::{api::ruma_api, UserId};

    ruma_api! {
        metadata: {
            description: "Set the display name of the user.",
            method: PUT,
            name: "set_display_name",
            r0_path: "/_matrix/client/r0/profile/:user_id/displayname",
            stable_path: "/_matrix/client/v3/profile/:user_id/displayname",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The user whose display name will be set.
            #[ruma_api(path)]
            pub user_id: &'a UserId,

            /// The new display name for the user.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub displayname: Option<&'a str>,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user ID and display name.
        pub fn new(user_id: &'a UserId, displayname: Option<&'a str>) -> Self {
            Self { user_id, displayname }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
