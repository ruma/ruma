//! Endpoints for room creation.

/// POST /_matrix/client/r0/createRoom
pub mod create_room {
    use ruma_identifiers::RoomId;

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub creation_content: Option<CreationContent>,
        pub invite: Option<Vec<String>>,
        pub name: Option<String>,
        pub preset: Option<RoomPreset>,
        pub room_alias_name: Option<String>,
        pub topic: Option<String>,
        pub visibility: Option<String>,
    }

    /// Extra options to be added to the `m.room.create` event.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct CreationContent {
        #[serde(rename="m.federate")]
        pub federate: Option<bool>,
    }

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// The response type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub room_id: RoomId,
    }

    /// A convenience parameter for setting a few default state events.
    #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
    pub enum RoomPreset {
        /// `join_rules` is set to `invite` and `history_visibility` is set to `shared`.
        #[serde(rename="private_chat")]
        PrivateChat,
        /// `join_rules` is set to `public` and `history_visibility` is set to `shared`.
        #[serde(rename="public_chat")]
        PublicChat,
        /// Same as `PrivateChat`, but all initial invitees get the same power level as the creator.
        #[serde(rename="trusted_private_chat")]
        TrustedPrivateChat,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = ();
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(_params: Self::PathParams) -> String {
            Self::router_path()
        }

        fn router_path() -> String {
            "/_matrix/client/r0/createRoom".to_string()
        }
    }
}
