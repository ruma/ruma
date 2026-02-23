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
        RoomId, UserId,
        api::{request, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::{
        StateEventType,
        room::{
            member::{MembershipState, RoomMemberEventContent, ThirdPartyInvite},
            third_party_invite::RoomThirdPartyInviteEventContent,
        },
    };

    use crate::{authentication::ServerSignatures, thirdparty::bind_callback};

    metadata! {
        method: PUT,
        rate_limited: false,
        authentication: ServerSignatures,
        path: "/_matrix/federation/v1/exchange_third_party_invite/{room_id}",
    }

    /// Request type for the `exchange_invite` endpoint.
    #[request]
    pub struct Request {
        /// The room ID to exchange the third-party invite in.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// The event type.
        ///
        /// Must be [`StateEventType::RoomMember`].
        #[serde(rename = "type")]
        pub kind: StateEventType,

        /// The user ID of the user who sent the original invite event.
        pub sender: UserId,

        /// The user ID of the invited user.
        pub state_key: UserId,

        /// The content of the invite event.
        ///
        /// It must have a `membership` of `invite` and the `third_party_invite` field must be set.
        pub content: Raw<RoomMemberEventContent>,
    }

    /// Response type for the `exchange_invite` endpoint.
    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` for a third-party invite exchange.
        pub fn new(
            room_id: RoomId,
            sender: UserId,
            state_key: UserId,
            content: Raw<RoomMemberEventContent>,
        ) -> Self {
            Self { room_id, kind: StateEventType::RoomMember, sender, state_key, content }
        }

        /// Creates a new `Request` for a third-party invite exchange from a `ThirdPartyInvite`.
        ///
        /// Returns an error if the serialization of the event content fails.
        pub fn with_third_party_invite(
            room_id: RoomId,
            sender: UserId,
            state_key: UserId,
            third_party_invite: ThirdPartyInvite,
        ) -> Result<Self, serde_json::Error> {
            let mut content = RoomMemberEventContent::new(MembershipState::Invite);
            content.third_party_invite = Some(third_party_invite);
            let content = Raw::new(&content)?;

            Ok(Self::new(room_id, sender, state_key, content))
        }

        /// Creates a new `Request` for a third-party invite exchange from a `ThirdPartyInvite` in
        /// the [`bind_callback::v1::Request`] and the matching
        /// [`RoomThirdPartyInviteEventContent`].
        ///
        /// Returns an error if the serialization of the event content fails.
        pub fn with_bind_callback_request_and_event(
            bind_callback_invite: bind_callback::v1::ThirdPartyInvite,
            room_third_party_invite_event: &RoomThirdPartyInviteEventContent,
        ) -> Result<Self, serde_json::Error> {
            let third_party_invite = ThirdPartyInvite::new(
                room_third_party_invite_event.display_name.clone(),
                bind_callback_invite.signed,
            );

            Self::with_third_party_invite(
                bind_callback_invite.room_id,
                bind_callback_invite.sender,
                bind_callback_invite.mxid,
                third_party_invite,
            )
        }
    }

    impl Response {
        /// Creates a new `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }
}
