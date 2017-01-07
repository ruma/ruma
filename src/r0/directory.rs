//! Endpoints for the public room directory.

/// [GET /_matrix/client/r0/publicRooms](https://matrix.org/docs/spec/client_server/r0.2.0.html#get-matrix-client-r0-publicrooms)
pub mod get_public_rooms {
    use ruma_identifiers::{RoomId, RoomAliasId};

    /// A chunk of the response, describing one room
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PublicRoomsChunk {
        pub world_readable: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub topic: Option<String>,
        pub num_joined_members: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<String>,
        pub room_id: RoomId,
        pub guest_can_join: bool,
        pub aliases: Vec<RoomAliasId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>
    }

    /// This API response type
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub start: String,
        pub chunk: Vec<PublicRoomsChunk>,
        pub end: String
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
    pub struct Endpoint;

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = ();
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Get
        }

        fn request_path(_params: Self::PathParams) -> String {
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/publicRooms"
        }

        fn name() -> &'static str {
            "get_public_rooms"
        }

        fn description() -> &'static str {
            "Get the list of rooms in this homeserver's public directory."
        }

        fn requires_authentication() -> bool {
            false
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
