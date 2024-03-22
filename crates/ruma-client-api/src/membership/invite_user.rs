//! `POST /_matrix/client/*/rooms/{roomId}/invite`
//!
//! Invite a user to a room.

pub mod v3 {
    //! `/v3/` ([spec (MXID)][spec-mxid], [spec (3PID)][spec-3pid])
    //!
    //! This endpoint has two forms: one to invite a user
    //! [by their Matrix identifier][spec-mxid], and one to invite a user
    //! [by their third party identifier][spec-3pid].
    //!
    //! [spec-mxid]: https://spec.matrix.org/v1.10/client-server-api/#post_matrixclientv3roomsroomidinvite
    //! [spec-3pid]: https://spec.matrix.org/v1.10/client-server-api/#post_matrixclientv3roomsroomidinvite-1

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedUserId,
    };
    use serde::{Deserialize, Serialize};

    use crate::membership::Invite3pid;

    const METADATA: Metadata = metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/:room_id/invite",
            1.1 => "/_matrix/client/v3/rooms/:room_id/invite",
        }
    };

    /// Request type for the `invite_user` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room where the user should be invited.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The user to invite.
        #[serde(flatten)]
        pub recipient: InvitationRecipient,

        /// Optional reason for inviting the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reason: Option<String>,
    }

    /// Response type for the `invite_user` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given room ID and invitation recipient.
        pub fn new(room_id: OwnedRoomId, recipient: InvitationRecipient) -> Self {
            Self { room_id, recipient, reason: None }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    /// Distinguishes between invititations by Matrix or third party identifiers.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[serde(untagged)]
    pub enum InvitationRecipient {
        /// Used to invite user by their Matrix identifier.
        UserId {
            /// Matrix identifier of user.
            user_id: OwnedUserId,
        },

        /// Used to invite user by a third party identifier.
        ThirdPartyId(Invite3pid),
    }

    #[cfg(test)]
    mod tests {
        use assert_matches2::assert_matches;
        use ruma_common::thirdparty::Medium;
        use serde_json::{from_value as from_json_value, json};

        use super::InvitationRecipient;

        #[test]
        fn deserialize_invite_by_user_id() {
            let incoming =
                from_json_value::<InvitationRecipient>(json!({ "user_id": "@carl:example.org" }))
                    .unwrap();

            assert_matches!(incoming, InvitationRecipient::UserId { user_id });
            assert_eq!(user_id, "@carl:example.org");
        }

        #[test]
        fn deserialize_invite_by_3pid() {
            let incoming = from_json_value::<InvitationRecipient>(json!({
                "id_server": "example.org",
                "id_access_token": "abcdefghijklmnop",
                "medium": "email",
                "address": "carl@example.org"
            }))
            .unwrap();

            assert_matches!(incoming, InvitationRecipient::ThirdPartyId(third_party_id));

            assert_eq!(third_party_id.id_server, "example.org");
            assert_eq!(third_party_id.id_access_token, "abcdefghijklmnop");
            assert_eq!(third_party_id.medium, Medium::Email);
            assert_eq!(third_party_id.address, "carl@example.org");
        }
    }
}
