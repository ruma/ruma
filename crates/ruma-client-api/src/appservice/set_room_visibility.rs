//! `PUT /_matrix/client/*/directory/list/appservice/{networkId}/{roomId}`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/application-service-api/#put_matrixclientv3directorylistappservicenetworkidroomid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, RoomId,
    };

    use crate::room::Visibility;

    const METADATA: Metadata = metadata! {
        description: "Updates the visibility of a given room on the application service's room directory.",
        method: PUT,
        name: "set_room_visibility",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/directory/list/appservice/:network_id/:room_id",
            1.1 => "/_matrix/client/v3/directory/list/appservice/:network_id/:room_id",
        }
    };

    #[request(error = crate::Error)]
    pub struct Request<'a> {
        /// The protocol (network) ID to update the room list for.
        #[ruma_api(path)]
        pub network_id: &'a str,

        /// The room ID to add to the directory.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// Whether the room should be visible (public) in the directory or not (private).
        pub visibility: Visibility,
    }

    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

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
