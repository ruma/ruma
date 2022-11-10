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
    //! [spec-mxid]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3roomsroomidinvite
    //! [spec-3pid]: https://spec.matrix.org/v1.4/client-server-api/#post_matrixclientv3roomsroomidinvite-1

    use ruma_common::{
        api::{request, response, Metadata},
        metadata,
        serde::Incoming,
        RoomId, UserId,
    };
    use serde::Serialize;

    use crate::membership::{IncomingInvite3pid, Invite3pid};

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
    pub struct Request<'a> {
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

    /// Response type for the `invite_user` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

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
        use ruma_common::thirdparty::Medium;
        use serde_json::{from_value as from_json_value, json};

        use super::IncomingInvitationRecipient;

        #[test]
        fn deserialize_invite_by_user_id() {
            let incoming = from_json_value::<IncomingInvitationRecipient>(
                json!({ "user_id": "@carl:example.org" }),
            )
            .unwrap();

            let user_id = assert_matches!(
                incoming,
                IncomingInvitationRecipient::UserId { user_id } => user_id
            );
            assert_eq!(user_id, "@carl:example.org");
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

            let third_party_id = assert_matches!(
                incoming,
                IncomingInvitationRecipient::ThirdPartyId(id) => id
            );

            assert_eq!(third_party_id.id_server, "example.org");
            assert_eq!(third_party_id.id_access_token, "abcdefghijklmnop");
            assert_eq!(third_party_id.medium, Medium::Email);
            assert_eq!(third_party_id.address, "carl@example.org");
        }
    }
}
