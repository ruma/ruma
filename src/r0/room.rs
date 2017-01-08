//! Endpoints for room creation.

/// [POST /_matrix/client/r0/createRoom](https://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-createroom)
pub mod create_room {
    use ruma_identifiers::{RoomId, UserId};

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub creation_content: Option<CreationContent>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        #[serde(default)]
        pub invite: Vec<UserId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub preset: Option<RoomPreset>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_alias_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub topic: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub visibility: Option<String>, // TODO: should be an enum ["public", "private"]
        // TODO: missing `invite_3pid`, `initial_state`
    }

    /// Extra options to be added to the `m.room.create` event.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct CreationContent {
        #[serde(rename="m.federate")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub federate: Option<bool>,
    }

    /// Details about this API endpoint.
    #[derive(Clone, Copy, Debug)]
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
            Self::router_path().to_string()
        }

        fn router_path() -> &'static str {
            "/_matrix/client/r0/createRoom"
        }

        fn name() -> &'static str {
            "create_room"
        }

        fn description() -> &'static str {
            "Create a new room."
        }

        fn requires_authentication() -> bool {
            true
        }

        fn rate_limited() -> bool {
            false
        }
    }
}
