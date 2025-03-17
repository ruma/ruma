//! `PUT /_matrix/federation/*/exchange_third_party_invite/{roomId}`
//!
//! The receiving server will verify the partial `m.room.member` event given in the request body.
//! If valid, the receiving server will issue an invite as per the [Inviting to a room] section
//! before returning a response to this request.
//!
//! [Inviting to a room]: https://spec.matrix.org/latest/server-server-api/#inviting-to-a-room

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv1exchange_third_party_inviteroomid

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedUserId,
    };
    use ruma_events::{
        room::member::{MembershipState, RoomMemberEventContent, ThirdPartyInvite},
        StateEventType,
    };

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/federation/v1/exchange_third_party_invite/:room_id",
        }
    };

    /// Request type for the `exchange_invite` endpoint.
    #[request]
    pub struct Request {
        /// The room ID to exchange the third-party invite in.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The event type.
        ///
        /// Must be [`StateEventType::RoomMember`].
        #[serde(rename = "type")]
        pub kind: StateEventType,

        /// The user ID of the user who sent the original invite event.
        pub sender: OwnedUserId,

        /// The user ID of the invited user.
        pub state_key: OwnedUserId,

        /// The content of the invite event.
        ///
        /// It must have a `membership` of `invite` and the `third_party_invite` field must be set.
        pub content: RoomMemberEventContent,
    }

    /// Response type for the `exchange_invite` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` for a third-party invite exchange.
        pub fn new(
            room_id: OwnedRoomId,
            sender: OwnedUserId,
            state_key: OwnedUserId,
            third_party_invite: ThirdPartyInvite,
        ) -> Self {
            let mut content = RoomMemberEventContent::new(MembershipState::Invite);
            content.third_party_invite = Some(third_party_invite);

            Self { room_id, kind: StateEventType::RoomMember, sender, state_key, content }
        }
    }

    impl Response {
        /// Creates a new `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
