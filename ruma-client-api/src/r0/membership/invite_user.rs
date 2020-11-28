//! [POST /_matrix/client/r0/rooms/{roomId}/invite][invite-by-user-id]
//!
//! This endpoint has two forms: one to invite a user
//! [by their Matrix identifier][invite-by-user-id], and one to invite a user
//! [by their third party identifier][invite-by-3pid].
//!
//! [invite-by-user-id]: https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-rooms-roomid-invite
//! [invite-by-3pid]: https://matrix.org/docs/spec/client_server/r0.6.0#id101

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};
use ruma_serde::Outgoing;
use serde::Serialize;

use super::{IncomingInvite3pid, Invite3pid};

ruma_api! {
    metadata: {
        description: "Invite a user to a room.",
        method: POST,
        name: "invite_user",
        path: "/_matrix/client/r0/rooms/:room_id/invite",
        rate_limited: true,
        authentication: AccessToken,
    }

    request: {
        /// The room where the user should be invited.
        #[ruma_api(path)]
        pub room_id: &'a RoomId,

        /// The user to invite.
        #[ruma_api(body)]
        pub recipient: InvitationRecipient<'a>,
    }

    #[derive(Default)]
    response: {}

    error: crate::Error
}

impl<'a> Request<'a> {
    /// Creates a new `Request` with the given room ID and invitation recipient.
    pub fn new(room_id: &'a RoomId, recipient: InvitationRecipient<'a>) -> Self {
        Self { room_id, recipient }
    }
}

impl Response {
    /// Creates an empty `Response`.
    pub fn new() -> Self {
        Self
    }
}

/// Distinguishes between invititations by Matrix or third party identifiers.
#[derive(Clone, Debug, PartialEq, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(PartialEq)]
#[serde(untagged)]
pub enum InvitationRecipient<'a> {
    /// Used to invite user by their Matrix identifer.
    UserId {
        /// Matrix identifier of user.
        user_id: &'a UserId,
    },

    /// Used to invite user by a third party identifer.
    ThirdPartyId(Invite3pid<'a>),
}

#[cfg(test)]
mod tests {
    use ruma_common::thirdparty::Medium;
    use ruma_identifiers::user_id;
    use serde_json::{from_value as from_json_value, json};

    use super::IncomingInvitationRecipient;
    use crate::r0::membership::IncomingInvite3pid;

    #[test]
    fn deserialize_invite_by_user_id() {
        let incoming = from_json_value::<IncomingInvitationRecipient>(
            json!({ "user_id": "@carl:example.org" }),
        )
        .unwrap();
        let user_id = user_id!("@carl:example.org");
        let recipient = IncomingInvitationRecipient::UserId { user_id };
        assert_eq!(incoming, recipient);
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
        let recipient = IncomingInvitationRecipient::ThirdPartyId(IncomingInvite3pid {
            id_server: "example.org".into(),
            id_access_token: "abcdefghijklmnop".into(),
            medium: Medium::Email,
            address: "carl@example.org".into(),
        });
        assert_eq!(incoming, recipient);
    }
}
