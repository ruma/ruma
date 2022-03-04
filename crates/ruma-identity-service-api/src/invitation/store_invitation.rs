//! `POST /_matrix/identity/*/store-invite`
//!
//! Endpoint to store pending invitations to a user's 3PID.

pub mod v2 {
    //! `/v2/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/identity-service-api/#post_matrixidentityv2store-invite

    use ruma_common::{
        api::ruma_api, room::RoomType, thirdparty::Medium, MxcUri, RoomAliasId, RoomId, RoomName,
        UserId,
    };
    use serde::{ser::SerializeSeq, Deserialize, Serialize};

    ruma_api! {
        metadata: {
            description: "Store pending invitations to a user's 3PID.",
            method: POST,
            name: "store_invitation",
            stable_path: "/_matrix/identity/v2/store-invite",
            authentication: AccessToken,
            rate_limited: false,
            added: 1.0,
        }

        request: {
            /// The type of the third party identifier for the invited user.
            ///
            /// Currently, only `Medium::Email` is supported.
            pub medium: &'a Medium,

            /// The email address of the invited user.
            pub address: &'a str,

            /// The Matrix room ID to which the user is invited.
            pub room_id: &'a RoomId,

            /// The Matrix user ID of the inviting user.
            pub sender: &'a UserId,

            /// The Matrix room alias for the room to which the user is invited.
            ///
            /// This should be retrieved from the `m.room.canonical` state event.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub room_alias: Option<&'a RoomAliasId>,

            /// The Content URI for the room to which the user is invited.
            ///
            /// This should be retrieved from the `m.room.avatar` state event.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub room_avatar_url: Option<&'a MxcUri>,

            /// The `join_rule` for the room to which the user is invited.
            ///
            /// This should be retrieved from the `m.room.join_rules` state event.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub room_join_rules: Option<&'a str>,

            /// The name of the room to which the user is invited.
            ///
            /// This should be retrieved from the `m.room.name` state event.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub room_name: Option<&'a RoomName>,

            /// The type of the room to which the user is invited.
            ///
            /// This should be retrieved from the `m.room.create` state event.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub room_type: Option<&'a RoomType>,

            /// The display name of the user ID initiating the invite.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub sender_display_name: Option<&'a str>,

            /// The Content URI for the avater of the user ID initiating the invite.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub sender_avatar_url: Option<&'a MxcUri>,
        }

        response: {
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
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request with the given medium, email address, room ID and sender.
        pub fn new(
            medium: &'a Medium,
            address: &'a str,
            room_id: &'a RoomId,
            sender: &'a UserId,
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
        pub fn email(address: &'a str, room_id: &'a RoomId, sender: &'a UserId) -> Self {
            Self::new(&Medium::Email, address, room_id, sender)
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
        pub server_key: String,

        /// The generated ephemeral public key.
        pub ephemeral_key: String,
    }

    impl<'de> Deserialize<'de> for PublicKeys {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let [server_key, ephemeral_key] = <[String; 2]>::deserialize(deserializer)?;

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
}
