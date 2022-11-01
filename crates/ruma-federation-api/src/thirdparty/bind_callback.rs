//! `PUT /_matrix/federation/*/3pid/onbind`
//!
//! Used by identity servers to notify the homeserver that one of its users has bound a third party
//! identifier successfully, including any pending room invites the identity server has been made
//! aware of.

pub mod v1 {
    //! `/v1/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.4/server-server-api/#put_matrixfederationv13pidonbind

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        thirdparty::Medium,
        OwnedRoomId, OwnedServerName, OwnedServerSigningKeyId, OwnedUserId, UserId,
    };
    use serde::{Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        description: "Used by identity servers to notify the homeserver that one of its users has bound a third party identifier successfully",
        method: PUT,
        name: "bind_callback",
        rate_limited: false,
        authentication: None,
        history: {
            1.0 => "/_matrix/federation/v1/3pid/onbind",
        }
    };

    #[request]
    pub struct Request<'a> {
        /// The type of third party identifier.
        ///
        /// Currently only `Medium::Email` is supported.
        pub medium: &'a Medium,

        /// The third party identifier itself.
        ///
        /// For example: an email address.
        pub address: &'a str,

        /// The user that is now bound to the third party identifier.
        pub mxid: &'a UserId,

        /// A list of pending invites that the third party identifier has received.
        pub invites: &'a [ThirdPartyInvite],
    }

    #[response]
    pub struct Response {}

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given medium, address, user ID and third party invites.
        pub fn new(
            medium: &'a Medium,
            address: &'a str,
            mxid: &'a UserId,
            invites: &'a [ThirdPartyInvite],
        ) -> Self {
            Self { medium, address, mxid, invites }
        }

        /// Creates a new `Request` with the given email address, user ID and third party invites.
        pub fn email(address: &'a str, mxid: &'a UserId, invites: &'a [ThirdPartyInvite]) -> Self {
            Self::new(&Medium::Email, address, mxid, invites)
        }
    }

    /// A pending invite the third party identifier has received.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
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

        /// Signature from the identity server using a long-term private key.
        pub signed: BTreeMap<OwnedServerName, BTreeMap<OwnedServerSigningKeyId, String>>,
    }

    impl ThirdPartyInvite {
        /// Creates a new third party invite with the given parameters.
        pub fn new(
            address: String,
            mxid: OwnedUserId,
            room_id: OwnedRoomId,
            sender: OwnedUserId,
            signed: BTreeMap<OwnedServerName, BTreeMap<OwnedServerSigningKeyId, String>>,
        ) -> Self {
            Self { medium: Medium::Email, address, mxid, room_id, sender, signed }
        }
    }
}
