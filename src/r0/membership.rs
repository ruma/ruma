//! Endpoints for room membership.

/// POST /_matrix/client/r0/rooms/{roomId}/join
pub mod join_by_room_id {
    use ruma_identifiers::RoomId;
    use ruma_signatures::Signatures;

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub third_party_signed: Option<ThirdPartySigned>,
    }

    /// A signature of an `m.third_party_invite` token to prove that this user owns a third party identity which has been invited to the room.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ThirdPartySigned {
        /// The state key of the m.third_party_invite event.
        pub token: String,
        /// A signatures object containing a signature of the entire signed object.
        pub signatures: Signatures,
        /// The Matrix ID of the invitee.
        pub mxid: String,
        /// The Matrix ID of the user who issued the invite.
        pub sender: String,
    }

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// The response type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Response {
        pub room_id: RoomId,
    }

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = Response;

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/join",
                params.room_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/join".to_string()
        }
    }
}

