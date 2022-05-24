//! `POST /_matrix/client/*/rooms/{roomId}/invite`

pub mod v3 {
    //! `/v3/` ([spec (MXID)][spec-mxid], [spec (3PID)][spec-3pid])
    //!
    //! This endpoint has two forms: one to invite a user
    //! [by their Matrix identifier][spec-mxid], and one to invite a user
    //! [by their third party identifier][spec-3pid].
    //!
    //! [spec-mxid]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidinvite
    //! [spec-3pid]: https://spec.matrix.org/v1.2/client-server-api/#post_matrixclientv3roomsroomidinvite-1

    use ruma_common::{api::ruma_api, serde::Incoming, RoomId, UserId};
    use serde::Serialize;

    use crate::membership::{IncomingInvite3pid, Invite3pid};

    ruma_api! {
        metadata: {
            description: "Invite a user to a room.",
            method: POST,
            name: "invite_user",
            r0_path: "/_matrix/client/r0/rooms/:room_id/invite",
            stable_path: "/_matrix/client/v3/rooms/:room_id/invite",
            rate_limited: true,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room where the user should be invited.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,

            /// The user to invite.
            #[serde(flatten)]
            pub recipient: InvitationRecipient<'a>,

            /// Optional reason for inviting the user.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub reason: Option<&'a str>,
        }

        #[derive(Default)]
        response: {}

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID and invitation recipient.
        pub fn new(room_id: &'a RoomId, recipient: InvitationRecipient<'a>) -> Self {
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
    #[derive(Clone, Debug, Incoming, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    #[serde(untagged)]
    pub enum InvitationRecipient<'a> {
        /// Used to invite user by their Matrix identifier.
        UserId {
            /// Matrix identifier of user.
            user_id: &'a UserId,
        },

        /// Used to invite user by a third party identifier.
        ThirdPartyId(Invite3pid<'a>),
    }

    #[cfg(test)]
    mod tests {
        use assert_matches::assert_matches;
        use ruma_common::{thirdparty::Medium, user_id};
        use serde_json::{from_value as from_json_value, json};

        use super::IncomingInvitationRecipient;

        #[test]
        fn deserialize_invite_by_user_id() {
            let incoming = from_json_value::<IncomingInvitationRecipient>(
                json!({ "user_id": "@carl:example.org" }),
            )
            .unwrap();

            assert_matches!(
                incoming,
                IncomingInvitationRecipient::UserId { user_id }
                if user_id == user_id!("@carl:example.org")
            );
        }

        #[test]
        fn deserialize_invite_by_3pid() {
            let incoming = from_json_value::<IncomingInvitationRecipient>(json!({
                "id_server": "example.org",
                "id_access_token": "abcdefghijklmnop",
                "medium": "email",
                "address": "carl@example.org"
            }))
            .unwrap();

            let third_party_id = match incoming {
                IncomingInvitationRecipient::UserId { .. } => panic!("wrong variant"),
                IncomingInvitationRecipient::ThirdPartyId(id) => id,
            };

            assert_eq!(third_party_id.id_server, "example.org");
            assert_eq!(third_party_id.id_access_token, "abcdefghijklmnop");
            assert_eq!(third_party_id.medium, Medium::Email);
            assert_eq!(third_party_id.address, "carl@example.org");
        }
    }
}
