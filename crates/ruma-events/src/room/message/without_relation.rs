use serde::Serialize;

use super::{MessageType, Relation, RoomMessageEventContent};
use crate::Mentions;

/// Form of [`RoomMessageEventContent`] without relation.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct RoomMessageEventContentWithoutRelation {
    /// A key which identifies the type of message being sent.
    ///
    /// This also holds the specific content of each message.
    #[serde(flatten)]
    pub msgtype: MessageType,

    /// The [mentions] of this event.
    ///
    /// [mentions]: https://spec.matrix.org/latest/client-server-api/#user-and-room-mentions
    #[serde(rename = "m.mentions", skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Mentions>,
}

impl RoomMessageEventContentWithoutRelation {
    /// Creates a new `RoomMessageEventContentWithoutRelation` with the given `MessageType`.
    pub fn new(msgtype: MessageType) -> Self {
        Self { msgtype, mentions: None }
    }

    /// Transform `self` into a `RoomMessageEventContent` with the given relation.
    pub fn with_relation(
        self,
        relates_to: Option<Relation<RoomMessageEventContentWithoutRelation>>,
    ) -> RoomMessageEventContent {
        let Self { msgtype, mentions } = self;
        RoomMessageEventContent { msgtype, relates_to, mentions }
    }
}

impl From<MessageType> for RoomMessageEventContentWithoutRelation {
    fn from(msgtype: MessageType) -> Self {
        Self::new(msgtype)
    }
}

impl From<RoomMessageEventContent> for RoomMessageEventContentWithoutRelation {
    fn from(value: RoomMessageEventContent) -> Self {
        let RoomMessageEventContent { msgtype, mentions, .. } = value;
        Self { msgtype, mentions }
    }
}
