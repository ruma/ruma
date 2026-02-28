//! `PUT /_matrix/client/*/rooms/{roomId}/typing/{userId}`
//!
//! Send a typing event to a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3roomsroomidtypinguserid

    use std::time::Duration;

    use ruma_common::{
        OwnedRoomId, OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
    };
    use serde::{Deserialize, Deserializer, Serialize, de::Error};

    metadata! {
        method: PUT,
        authentication: AccessToken,
        rate_limited: true,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/typing/{user_id}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/typing/{user_id}",
        }
    }

    /// Request type for the `create_typing_event` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The room in which the user is typing.
        #[ruma_api(path)]
        pub room_id: OwnedRoomId,

        /// The user who has started to type.
        #[ruma_api(path)]
        pub user_id: OwnedUserId,

        /// Whether the user is typing within a length of time or not.
        #[ruma_api(body)]
        pub state: Typing,
    }

    /// Response type for the `create_typing_event` endpoint.
    #[response(error = crate::Error)]
    #[derive(Default)]
    pub struct Response {}

    impl Request {
        /// Creates a new `Request` with the given user ID, room ID and typing state.
        pub fn new(user_id: OwnedUserId, room_id: OwnedRoomId, state: Typing) -> Self {
            Self { user_id, room_id, state }
        }
    }

    impl Response {
        /// Creates an empty `Response`.
        pub fn new() -> Self {
            Self {}
        }
    }

    /// A mark for whether the user is typing or not.
    #[derive(Clone, Copy, Debug)]
    #[allow(clippy::exhaustive_enums)]
    pub enum Typing {
        /// The user is currently not typing.
        No,

        /// The user is currently typing.
        Yes(TypingInfo),
    }

    impl From<TypingInfo> for Typing {
        fn from(value: TypingInfo) -> Self {
            Self::Yes(value)
        }
    }

    /// Details about the user currently typing.
    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct TypingInfo {
        /// The length of time to mark this user as typing.
        pub timeout: Duration,
    }

    impl TypingInfo {
        /// Create a new `TypingInfo` with the given timeout.
        pub fn new(timeout: Duration) -> Self {
            Self { timeout }
        }
    }

    #[derive(Deserialize, Serialize)]
    struct TypingSerdeRepr {
        typing: bool,

        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        timeout: Option<Duration>,
    }

    impl Serialize for Typing {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let repr = match self {
                Self::No => TypingSerdeRepr { typing: false, timeout: None },
                Self::Yes(TypingInfo { timeout }) => {
                    TypingSerdeRepr { typing: true, timeout: Some(*timeout) }
                }
            };

            repr.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Typing {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let repr = TypingSerdeRepr::deserialize(deserializer)?;

            Ok(if repr.typing {
                Typing::Yes(TypingInfo {
                    timeout: repr.timeout.ok_or_else(|| D::Error::missing_field("timeout"))?,
                })
            } else {
                Typing::No
            })
        }
    }
}
