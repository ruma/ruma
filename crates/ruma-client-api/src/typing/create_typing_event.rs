//! `PUT /_matrix/client/*/rooms/{roomId}/typing/{userId}`
//!
//! Send a typing event to a room.

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/latest/client-server-api/#put_matrixclientv3roomsroomidtypinguserid

    use std::time::Duration;

    use ruma_common::{
        api::{request, response, Metadata},
        metadata, OwnedRoomId, OwnedUserId,
    };
    use serde::{de::Error, Deserialize, Deserializer, Serialize};

    const METADATA: Metadata = metadata! {
        method: PUT,
        authentication: AccessToken,
        rate_limited: true,
        history: {
            1.0 => "/_matrix/client/r0/rooms/{room_id}/typing/{user_id}",
            1.1 => "/_matrix/client/v3/rooms/{room_id}/typing/{user_id}",
        }
    };

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

    /// A mark for whether the user is typing within a length of time or not.
    #[derive(Clone, Copy, Debug, Serialize)]
    #[serde(into = "TypingInner")]
    #[allow(clippy::exhaustive_enums)]
    pub enum Typing {
        /// Not typing.
        No,

        /// Typing during the specified length of time.
        Yes(Duration),
    }

    #[derive(Deserialize, Serialize)]
    struct TypingInner {
        typing: bool,

        #[serde(
            with = "ruma_common::serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        timeout: Option<Duration>,
    }

    impl From<Typing> for TypingInner {
        fn from(typing: Typing) -> Self {
            match typing {
                Typing::No => Self { typing: false, timeout: None },
                Typing::Yes(time) => Self { typing: true, timeout: Some(time) },
            }
        }
    }

    impl<'de> Deserialize<'de> for Typing {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let inner = TypingInner::deserialize(deserializer)?;

            match (inner.typing, inner.timeout) {
                (false, _) => Ok(Self::No),
                (true, Some(time)) => Ok(Self::Yes(time)),
                _ => Err(D::Error::missing_field("timeout")),
            }
        }
    }
}
