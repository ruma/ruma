//! `GET /_matrix/client/*/user/mutual_rooms/{user_id}`

pub mod unstable {
    //! `/unstable/` ([spec])
    //!
    //! [spec]: https://github.com/matrix-org/matrix-spec-proposals/blob/hs/shared-rooms/proposals/2666-get-rooms-in-common.md

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, UserId,
    };

    const METADATA: Metadata = metadata! {
        description: "Get mutual rooms with another user.",
        method: GET,
        name: "mutual_rooms",
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable => "/_matrix/client/unstable/uk.half-shot.msc2666/user/mutual_rooms/:user_id",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The user to search mutual rooms for.
        #[ruma_api(path)]
        pub user_id: &'a UserId,
    }

    #[response(error = crate::Error)]
    pub struct Response {
        /// A list of rooms the user is in together with the authenticated user.
        pub joined: Vec<OwnedRoomId>,
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
