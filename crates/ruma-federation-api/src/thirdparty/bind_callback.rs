//! `PUT /_matrix/federation/*/3pid/onbind`
//!
//! Used by identity servers to notify the homeserver that one of its users has bound a third party
//! identifier successfully, including any pending room invites the identity server has been made
//! aware of.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/server-server-api/#put_matrixfederationv13pidonbind

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Medium,
        OwnedRoomId, OwnedUserId,
    };
    use ruma_events::room::member::SignedContent;
    use serde::{Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        method: PUT,
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/federation/v1/3pid/onbind",
        }
    };

    /// Request type for the `bind_callback` endpoint.
    #[request]
    pub struct Request {
        /// The type of third party identifier.
        ///
        /// Currently only `Medium::Email` is supported.
        pub medium: Medium,

        /// The third party identifier itself.
        ///
        /// For example: an email address.
        pub address: String,

        /// The user that is now bound to the third party identifier.
        pub mxid: OwnedUserId,

        /// A list of pending invites that the third party identifier has received.
        pub invites: Vec<ThirdPartyInvite>,
    }

    /// Response type for the `bind_callback` endpoint.
    #[response]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given medium, address, user ID and third party invites.
        pub fn new(
            medium: Medium,
            address: String,
            mxid: OwnedUserId,
            invites: Vec<ThirdPartyInvite>,
        ) -> Self {
            Self { medium, address, mxid, invites }
        }

        /// Creates a new `Request` with the given email address, user ID and third party invites.
        pub fn email(address: String, mxid: OwnedUserId, invites: Vec<ThirdPartyInvite>) -> Self {
            Self::new(Medium::Email, address, mxid, invites)
        }
    }

    /// A pending invite the third party identifier has received.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThirdPartyInvite {
        /// The type of third party invite issues.
        ///
        /// Currently only `Medium::Email` is used.
        pub medium: Medium,

        /// The third party identifier that received the invite.
        pub address: String,

        /// The now-bound user ID that received the invite.
        pub mxid: OwnedUserId,

        /// The room ID the invite is valid for.
        pub room_id: OwnedRoomId,

        /// The user ID that sent the invite.
        pub sender: OwnedUserId,

        /// A block of content which has been signed, which servers can use to verify the
        /// third-party invite.
        pub signed: SignedContent,
    }

    impl ThirdPartyInvite {
        /// Creates a new third party invite with the given parameters.
        pub fn new(
            address: String,
            mxid: OwnedUserId,
            room_id: OwnedRoomId,
            sender: OwnedUserId,
            signed: SignedContent,
        ) -> Self {
            Self { medium: Medium::Email, address, mxid, room_id, sender, signed }
        }
    }
}
