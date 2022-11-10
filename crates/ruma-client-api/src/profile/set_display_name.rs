//! `PUT /_matrix/client/*/profile/{userId}/displayname`
//!
//! Set the display name of the user.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/client-server-api/#put_matrixclientv3profileuseriddisplayname

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, UserId,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/profile/:user_id/displayname",
            1.1 => "/_matrix/client/v3/profile/:user_id/displayname",
        }
    };

    /// Request type for the `set_display_name` endpoint.
    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The user whose display name will be set.
        #[ruma_api(path)]
        pub user_id: &'a UserId,

        /// The new display name for the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<&'a str>,
    }

    /// Response type for the `set_display_name` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

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
