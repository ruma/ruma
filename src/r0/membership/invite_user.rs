//! [POST /_matrix/client/r0/rooms/{roomId}/invite][invite-by-user-id]
//!
//! This endpoint has two forms: one to invite a user
//! [by their Matrix identifier][invite-by-user-id], and one to invite a user
//! [by their third party identifier][invite-by-3pid].
//!
//! [invite-by-user-id]: https://matrix.org/docs/spec/client_server/r0.6.0.html#post-matrix-client-r0-rooms-roomid-invite
//! [invite-by-3pid]: https://matrix.org/docs/spec/client_server/r0.6.0#id101
use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};
use serde::{Deserialize, Serialize};

use super::Invite3pid;

ruma_api! {
    metadata {
        description: "Invite a user to a room.",
        method: POST,
        name: "invite_user",
        path: "/_matrix/client/r0/rooms/:room_id/invite",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The room where the user should be invited.
        #[ruma_api(path)]
        pub room_id: RoomId,
        /// The user to invite.
        #[ruma_api(body)]
        pub recipient: InvitationRecipient,
    }

    response {}

    error: crate::Error
}

/// Distinguishes between invititations by Matrix or third party identifiers.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum InvitationRecipient {
    /// Used to invite user by their Matrix identifer.
    UserId {
        /// Matrix identifier of user.
        user_id: UserId,
    },
    /// Used to invite user by a third party identifer.
    ThirdPartyId(Invite3pid),
}

#[cfg(test)]
mod tests {
    use super::InvitationRecipient;
    use crate::r0::{membership::Invite3pid, thirdparty::Medium};
    use ruma_identifiers::UserId;
    use std::convert::TryFrom;
    #[test]
    fn deserialize_invite_by_user_id() {
        let incoming =
            serde_json::from_str::<InvitationRecipient>(r#" { "user_id": "@carl:example.org" } "#)
                .unwrap();
        let user_id = UserId::try_from("@carl:example.org").unwrap();
        let recipient = InvitationRecipient::UserId { user_id };
        assert_eq!(incoming, recipient);
    }

    #[test]
    fn deserialize_invite_by_3pid() {
        let incoming = serde_json::from_str::<InvitationRecipient>(
            r#"
                {
                    "id_server": "example.org",
                    "id_access_token": "abcdefghijklmnop",
                    "medium": "email",
                    "address": "carl@example.org"
                }
                "#,
        )
        .unwrap();
        let recipient = InvitationRecipient::ThirdPartyId(Invite3pid {
            id_server: "example.org".to_string(),
            id_access_token: "abcdefghijklmnop".to_string(),
            medium: Medium::Email,
            address: "carl@example.org".to_string(),
        });
        assert_eq!(incoming, recipient);
    }
}
