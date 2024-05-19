//! `PUT /_matrix/client/*/directory/list/appservice/{networkId}/{roomId}`
//!
//! Updates the visibility of a given room on the application service's room directory.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/application-service-api/#put_matrixclientv3directorylistappservicenetworkidroomid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId,
    };

    use crate::room::Visibility;

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/directory/list/appservice/{network_id}/{room_id}",
            1.1 => "/_matrix/client/v3/directory/list/appservice/{network_id}/{room_id}",
        }
    };

    /// Request type for the `set_room_visibility` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The protocol (network) ID to update the room list for.
        #[ruma_api(path)]
        pub network_id: String,

        /// The room ID to add to the directory.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// Whether the room should be visible (public) in the directory or not (private).
        pub visibility: Visibility,
    }

    /// Response type for the `set_room_visibility` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given network ID, room ID and visibility.
        pub fn new(network_id: String, room_id: OwnedRoomId, visibility: Visibility) -> Self {
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
