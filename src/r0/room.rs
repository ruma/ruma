//! Endpoints for room creation.

/// POST /_matrix/client/r0/createRoom
pub mod create_room {
    use ruma_identifiers::RoomId;

    pub const HTTP_METHOD: &'static str = "POST";
    pub const PATH: &'static str = "/_matrix/client/r0/createRoom";

    /// Extra options to be added to the `m.room.create` event.
    #[derive(Clone, Debug, Deserialize)]
    pub struct CreationContent {
        #[serde(rename="m.federate")]
        pub federate: Option<bool>,
    }

    /// The request type.
    #[derive(Clone, Debug, Deserialize)]
    pub struct Request {
        pub creation_content: Option<CreationContent>,
        pub invite: Option<Vec<String>>,
        pub name: Option<String>,
        pub preset: Option<RoomPreset>,
        pub room_alias_name: Option<String>,
        pub topic: Option<String>,
        pub visibility: Option<String>,
    }

    /// The response type.
    #[derive(Debug, Serialize)]
    pub struct Response {
        room_id: RoomId,
    }

    /// A convenience parameter for setting a few default state events.
    #[derive(Clone, Copy, Debug, Deserialize)]
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
}
