//! `GET /_matrix/client/*/user/mutual_rooms/{user_id}`

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/blob/hs/shared-rooms/proposals/2666-get-rooms-in-common.md

    use ruma_common::{api::ruma_api, OwnedRoomId, UserId};

    ruma_api! {
        metadata: {
            description: "Get mutual rooms with another user.",
            method: GET,
            name: "mutual_rooms",
            unstable_path: "/_matrix/client/unstable/uk.half-shot.msc2666/user/mutual_rooms/:user_id",
            rate_limited: true,
            authentication: AccessToken,
        }

        request: {
            /// The user to search mutual rooms for.
            #[ruma_api(path)]
            pub user_id: &'a UserId,
        }

        response: {
            /// A list of rooms the user is in together with the authenticated user.
            pub joined: Vec<OwnedRoomId>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given user id.
        pub fn new(user_id: &'a UserId) -> Self {
            Self { user_id }
        }
    }

    impl Response {
        /// Creates a `Response` with the given room ids.
        pub fn new(joined: Vec<OwnedRoomId>) -> Self {
            Self { joined }
        }
    }
}
