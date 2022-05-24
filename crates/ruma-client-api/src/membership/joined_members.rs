//! `GET /_matrix/client/*/rooms/{roomId}/joined_members`

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.2/client-server-api/#get_matrixclientv3roomsroomidjoined_members

    use std::collections::BTreeMap;

    use ruma_common::{api::ruma_api, OwnedMxcUri, OwnedUserId, RoomId};
    use serde::{Deserialize, Serialize};

    ruma_api! {
        metadata: {
            description: "Get a map of user ids to member info objects for members of the room. Primarily for use in Application Services.",
            method: GET,
            name: "joined_members",
            r0_path: "/_matrix/client/r0/rooms/:room_id/joined_members",
            stable_path: "/_matrix/client/v3/rooms/:room_id/joined_members",
            rate_limited: false,
            authentication: AccessToken,
            added: 1.0,
        }

        request: {
            /// The room to get the members of.
            #[ruma_api(path)]
            pub room_id: &'a RoomId,
        }

        response: {
            /// A list of the rooms the user is in, i.e.
            /// the ID of each room in which the user has joined membership.
            pub joined: BTreeMap<OwnedUserId, RoomMember>,
        }

        error: crate::Error
    }

    impl<'a> Request<'a> {
        /// Creates a new `Request` with the given room ID.
        pub fn new(room_id: &'a RoomId) -> Self {
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
        /// If you activate the `compat` feature, this field being an empty string in JSON will
        /// result in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat",
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
    mod test {
        use assert_matches::assert_matches;
        use serde_json::{from_value as from_json_value, json};

        use super::RoomMember;

        #[test]
        fn deserialize_room_member() {
            assert_matches!(
                from_json_value::<RoomMember>(json!({
                    "display_name": "alice",
                    "avatar_url": "mxc://localhost/wefuiwegh8742w",
                })).unwrap(),
                RoomMember {
                    display_name: Some(display_name),
                    avatar_url: Some(avatar_url),
                } if display_name == "alice"
                    && avatar_url == "mxc://localhost/wefuiwegh8742w"
            );

            #[cfg(feature = "compat")]
            assert_matches!(
                from_json_value::<RoomMember>(json!({
                    "display_name": "alice",
                    "avatar_url": "",
                })).unwrap(),
                RoomMember {
                    display_name: Some(display_name),
                    avatar_url: None,
                } if display_name == "alice"
            );
        }
    }
}
