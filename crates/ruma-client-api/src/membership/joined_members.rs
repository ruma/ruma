//! `GET /_matrix/client/*/rooms/{roomId}/joined_members`
//!
//! Get a map of user IDs to member info objects for members of the room. Primarily for use in
//! Application Services.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#get_matrixclientv3roomsroomidjoined_members

    use std::collections::BTreeMap;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedMxcUri, OwnedRoomId, OwnedUserId,
    };
    use serde::{Deserialize, Serialize};

    const METADATA: Metadata = metadata! {
        method: GET,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/joined_members",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/joined_members",
        }
    };

    /// Request type for the `joined_members` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room to get the members of.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,
    }

    /// Response type for the `joined_members` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A list of the rooms the user is in, i.e.
        /// the ID of each room in which the user has joined membership.
        pub joined: BTreeMap<OwnedUserId, RoomMember>,
    }

    impl Request {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: OwnedRoomId) -> Self {
            Self { room_id }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given joined rooms.
        pub fn new(joined: BTreeMap<OwnedUserId, RoomMember>) -> Self {
            Self { joined }
        }
    }

    /// Information about a room member.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
    pub struct RoomMember {
        /// The display name of the user.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub display_name: Option<String>,

        /// The mxc avatar url of the user.
        ///
        /// If you activate the `compat-empty-string-null` feature, this field being an empty
        /// string in JSON will result in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat-empty-string-null",
            serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
        )]
        pub avatar_url: Option<OwnedMxcUri>,
    }

    impl RoomMember {
        /// Creates an empty `RoomMember`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    #[cfg(test)]
    mod tests {
        use ruma_common::mxc_uri;
        use serde_json::{from_value as from_json_value, json};

        use super::RoomMember;

        #[test]
        fn deserialize_room_member() {
            let member = from_json_value::<RoomMember>(json!({
                "display_name": "alice",
                "avatar_url": "mxc://localhost/wefuiwegh8742w",
            }))
            .unwrap();
            assert_eq!(member.display_name.as_deref(), Some("alice"));
            assert_eq!(
                member.avatar_url.as_deref(),
                Some(mxc_uri!("mxc://localhost/wefuiwegh8742w"))
            );

            #[cfg(feature = "compat-empty-string-null")]
            {
                let member = from_json_value::<RoomMember>(json!({
                    "display_name": "alice",
                    "avatar_url": "",
                }))
                .unwrap();
                assert_eq!(member.display_name.as_deref(), Some("alice"));
                assert_eq!(member.avatar_url, None);
            }
        }
    }
}
