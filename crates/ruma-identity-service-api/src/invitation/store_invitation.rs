//! `POST /_matrix/identity/*/store-invite`
//!
//! Store pending invitations to a user's third-party ID.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/identity-service-api/#post_matrixidentityv2store-invite

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        room::RoomType,
        third_party_invite::IdentityServerBase64PublicKey,
        thirdparty::Medium,
        OwnedMxcUri, OwnedRoomAliasId, OwnedRoomId, OwnedUserId,
    };
    use ruma_events::room::third_party_invite::RoomThirdPartyInviteEventContent;
    use serde::{ser::SerializeSeq, Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/identity/v2/store-invite",
        }
    };

    /// Request type for the `store_invitation` endpoint.
    #[request]
    pub struct Request {
        /// The type of the third party identifier for the invited user.
        ///
        /// Currently, only `Medium::Email` is supported.
        pub medium: Medium,

        /// The email address of the invited user.
        pub address: String,

        /// The Matrix room ID to which the user is invited.
        pub room_id: OwnedRoomId,

        /// The Matrix user ID of the inviting user.
        pub sender: OwnedUserId,

        /// The Matrix room alias for the room to which the user is invited.
        ///
        /// This should be retrieved from the `m.room.canonical` state event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_alias: Option<OwnedRoomAliasId>,

        /// The Content URI for the room to which the user is invited.
        ///
        /// This should be retrieved from the `m.room.avatar` state event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_avatar_url: Option<OwnedMxcUri>,

        /// The `join_rule` for the room to which the user is invited.
        ///
        /// This should be retrieved from the `m.room.join_rules` state event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_join_rules: Option<String>,

        /// The name of the room to which the user is invited.
        ///
        /// This should be retrieved from the `m.room.name` state event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_name: Option<String>,

        /// The type of the room to which the user is invited.
        ///
        /// This should be retrieved from the `m.room.create` state event.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_type: Option<RoomType>,

        /// The display name of the user ID initiating the invite.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sender_display_name: Option<String>,

        /// The Content URI for the avater of the user ID initiating the invite.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sender_avatar_url: Option<OwnedMxcUri>,
    }

    /// Response type for the `store_invitation` endpoint.
    #[response]
    pub struct Response {
        /// The generated token.
        ///
        /// Must be a string consisting of the characters `[0-9a-zA-Z.=_-]`. Its length must not
        /// exceed 255 characters and it must not be empty.
        pub token: String,

        /// A list of [server's long-term public key, generated ephemeral public key].
        pub public_keys: PublicKeys,

        /// The generated (redacted) display_name.
        ///
        /// An example is `f...@b...`.
        pub display_name: String,
    }

    impl Request {
        /// Creates a new `Request with the given medium, email address, room ID and sender.
        pub fn new(
            medium: Medium,
            address: String,
            room_id: OwnedRoomId,
            sender: OwnedUserId,
        ) -> Self {
            Self {
                medium,
                address,
                room_id,
                sender,
                room_alias: None,
                room_avatar_url: None,
                room_join_rules: None,
                room_name: None,
                room_type: None,
                sender_display_name: None,
                sender_avatar_url: None,
            }
        }

        /// Creates a new `Request` with the given email address, room ID and sender.
        pub fn email(address: String, room_id: OwnedRoomId, sender: OwnedUserId) -> Self {
            Self::new(Medium::Email, address, room_id, sender)
        }
    }

    impl Response {
        /// Creates a new `Response` with the given token, public keys and display name.
        pub fn new(token: String, public_keys: PublicKeys, display_name: String) -> Self {
            Self { token, public_keys, display_name }
        }
    }

    /// The server's long-term public key and generated ephemeral public key.
    #[derive(Debug, Clone)]
    #[allow(clippy::exhaustive_structs)]
    pub struct PublicKeys {
        /// The server's long-term public key.
        pub server_key: PublicKey,

        /// The generated ephemeral public key.
        pub ephemeral_key: PublicKey,
    }

    impl<'de> Deserialize<'de> for PublicKeys {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let [server_key, ephemeral_key] = <[PublicKey; 2]>::deserialize(deserializer)?;

            Ok(Self { server_key, ephemeral_key })
        }
    }

    impl Serialize for PublicKeys {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(2))?;

            seq.serialize_element(&self.server_key)?;
            seq.serialize_element(&self.ephemeral_key)?;

            seq.end()
        }
    }

    /// A server's long-term or ephemeral public key.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[non_exhaustive]
    pub struct PublicKey {
        /// The public key, encoded using unpadded base64.
        pub public_key: IdentityServerBase64PublicKey,

        /// The URI of an endpoint where the validity of this key can be checked by passing it as a
        /// `public_key` query parameter.
        pub key_validity_url: String,
    }

    impl PublicKey {
        /// Constructs a new `PublicKey` with the given encoded public key and key validity URL.
        pub fn new(public_key: IdentityServerBase64PublicKey, key_validity_url: String) -> Self {
            Self { public_key, key_validity_url }
        }
    }

    impl From<PublicKey> for ruma_events::room::third_party_invite::PublicKey {
        fn from(key: PublicKey) -> Self {
            let mut new_key = Self::new(key.public_key);
            new_key.key_validity_url = Some(key.key_validity_url);
            new_key
        }
    }

    impl From<Response> for RoomThirdPartyInviteEventContent {
        fn from(response: Response) -> Self {
            let mut content = RoomThirdPartyInviteEventContent::new(
                response.display_name,
                response.public_keys.server_key.key_validity_url.clone(),
                response.public_keys.server_key.public_key.clone(),
            );
            content.public_keys = Some(vec![
                response.public_keys.server_key.into(),
                response.public_keys.ephemeral_key.into(),
            ]);
            content
        }
    }
}
