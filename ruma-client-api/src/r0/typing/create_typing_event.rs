//! [PUT /_matrix/client/r0/rooms/{roomId}/typing/{userId}](https://matrix.org/docs/spec/client_server/r0.6.0#put-matrix-client-r0-rooms-roomid-typing-userid)

use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::time::Duration;

use ruma_api::ruma_api;
use ruma_identifiers::{RoomId, UserId};

ruma_api! {
    metadata: {
        method: PUT,
        path: "/_matrix/client/r0/rooms/:room_id/typing/:user_id",
        name: "create_typing_event",
        description: "Send a typing event to a room.",
        requires_authentication: true,
        rate_limited: true,
    }

    request: {
        /// The user who has started to type.
        #[ruma_api(path)]
        pub user_id: UserId,

        /// The room in which the user is typing.
        #[ruma_api(path)]
        pub room_id: RoomId,

        /// Whether the user is typing within a length of time or not.
        #[serde(flatten)]
        pub state: Typing,
    }

    response: {}

    error: crate::Error
}

/// A mark for whether the user is typing within a length of time or not.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(into = "TypingInner")]
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
        with = "ruma_serde::duration::opt_ms",
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
