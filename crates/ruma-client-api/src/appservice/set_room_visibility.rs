//! `PUT /_matrix/client/*/directory/list/appservice/{networkId}/{roomId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/application-service-api/#put_matrixclientv3directorylistappservicenetworkidroomid

    use ruma_common::{api::ruma_api, RoomId};

    use crate::room::Visibility;

    ruma_api! {
        metadata: {
            description: "Updates the visibility of a given room on the application service's room directory.",
            method: PUT,
            name: "set_room_visibility",
            r0_path: "/_matrix/client/r0/directory/list/appservice/:network_id/:room_id",
            stable_path: "/_matrix/client/v3/directory/list/appservice/:network_id/:room_id",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The protocol (network) ID to update the room list for.
            #[ruma_api(path)]
            pub network_id: &'a str,

            /// The room ID to add to the directory.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// Whether the room should be visible (public) in the directory or not (private).
            pub visibility: Visibility,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given network ID, room ID and visibility.
        pub fn new(network_id: &'a str, room_id: &'a RoomId, visibility: Visibility) -> Self {
            Self { network_id, room_id, visibility }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
