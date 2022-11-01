//! `PUT /_matrix/federation/*/exchange_third_party_invite/{roomId}`
//!
//! The receiving server will verify the partial `m.room.member` event given in the request body.
//! If valid, the receiving server will issue an invite as per the [Inviting to a room] section
//! before returning a response to this request.
//!
//! [Inviting to a room]: https://spec.matrix.org/v1.4/server-server-api/#inviting-to-a-room

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/server-server-api/#put_matrixfederationv1exchange_third_party_inviteroomid

    use ruma_common::{
        api::{request, response, Metadata},
        events::{room::member::ThirdPartyInvite, StateEventType},
        metadata, RoomId, UserId,
    };

    const METADATA: Metadata = metadata! {
        description: "The receiving server will verify the partial m.room.member event given in the request body.",
        method: PUT,
        name: "exchange_invite",
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/federation/v1/exchange_third_party_invite/:room_id",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The room ID to exchange a third party invite in.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The event type.
        ///
        /// Must be `StateEventType::RoomMember`.
        #[serde(rename = "type")]
        pub kind: StateEventType,

        /// The user ID of the user who sent the original invite event.
        pub sender: &'a UserId,

        /// The user ID of the invited user.
        pub state_key: &'a UserId,

        /// The content of the invite event.
        pub content: &'a ThirdPartyInvite,
    }

    #[response]
    #[derive(Default)]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` for a third party invite exchange
        pub fn new(
            room_id: &'a RoomId,
            sender: &'a UserId,
            state_key: &'a UserId,
            content: &'a ThirdPartyInvite,
        ) -> Self {
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
