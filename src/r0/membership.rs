//! Endpoints for room membership.

use ruma_signatures::Signatures;

/// A signature of an `m.third_party_invite` token to prove that this user owns a third party identity which has been invited to the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThirdPartySigned {
    /// The Matrix ID of the invitee.
    pub mxid: String,
    /// The Matrix ID of the user who issued the invite.
    pub sender: String,
    /// A signatures object containing a signature of the entire signed object.
    pub signatures: Signatures,
    /// The state key of the m.third_party_invite event.
    pub token: String,
}

/// POST /_matrix/client/r0/rooms/{roomId}/invite
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-invite)
pub mod invite {
    use ruma_identifiers::RoomId;

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub user_id: String,
    }

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/invite",
                params.room_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/invite".to_string()
        }
    }
}

/// POST /_matrix/client/r0/join/{roomIdOrAlias}
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-join-roomidoralias)
pub mod join_by_room_id_or_alias {
    use ruma_identifiers::{RoomId, RoomIdOrAliasId};
    use super::ThirdPartySigned;

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub third_party_signed: Option<ThirdPartySigned>,
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
        pub room_id_or_alias: RoomIdOrAliasId,
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
            match params.room_id_or_alias {
                RoomIdOrAliasId::RoomId(room_id) => {
                    format!(
                        "/_matrix/client/r0/join/{}",
                        room_id
                    )
                }
                RoomIdOrAliasId::RoomAliasId(room_alias_id) => {
                    format!(
                        "/_matrix/client/r0/join/{}",
                        room_alias_id
                    )
                }
            }
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id_or_alias/join".to_string()
        }
    }
}

/// POST /_matrix/client/r0/rooms/{roomId}/join
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-join)
pub mod join_by_room_id {
    use ruma_identifiers::RoomId;
    use super::ThirdPartySigned;

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub third_party_signed: Option<ThirdPartySigned>,
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

/// POST /_matrix/client/r0/rooms/{roomId}/forget
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-forget)
pub mod forget {
    use ruma_identifiers::RoomId;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/forget",
                params.room_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/forget".to_string()
        }
    }
}

/// POST /_matrix/client/r0/rooms/{roomId}/leave
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-leave)
pub mod leave {
    use ruma_identifiers::RoomId;

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = ();
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/leave",
                params.room_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/leave".to_string()
        }
    }
}

/// POST /_matrix/client/r0/rooms/{roomId}/kick
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-kick)
pub mod kick {
    use ruma_identifiers::RoomId;

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub user_id: String,
        pub reason: Option<String>,
    }

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/kick",
                params.room_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/kick".to_string()
        }
    }
}

/// POST /_matrix/client/r0/rooms/{roomId}/unban
///
/// [matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-unban)
pub mod unban {
    use ruma_identifiers::RoomId;

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub user_id: String,
    }

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/unban",
                params.room_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/unban".to_string()
        }
    }
}

/// POST /_matrix/client/r0/rooms/{roomId}/ban
///
/// [Matrix spec link](http://matrix.org/docs/spec/client_server/r0.2.0.html#post-matrix-client-r0-rooms-roomid-ban)
pub mod ban {
    use ruma_identifiers::RoomId;

    /// The request type.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BodyParams {
        pub reason: Option<String>,
        pub user_id: String,
    }

    /// Details about this API endpoint.
    pub struct Endpoint;

    /// This API endpoint's path parameters.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub room_id: RoomId,
    }

    impl ::Endpoint for Endpoint {
        type BodyParams = BodyParams;
        type PathParams = PathParams;
        type QueryParams = ();
        type Response = ();

        fn method() -> ::Method {
            ::Method::Post
        }

        fn request_path(params: Self::PathParams) -> String {
            format!(
                "/_matrix/client/r0/rooms/{}/ban",
                params.room_id
            )
        }

        fn router_path() -> String {
            "/_matrix/client/r0/rooms/:room_id/ban".to_string()
        }
    }
}
