use serde::Serialize;

use super::{
    AddMentions, ForwardThread, MessageType, Relation, ReplacementMetadata, ReplyMetadata,
    ReplyWithinThread, RoomMessageEventContent,
};
use crate::{
    relation::{InReplyTo, Replacement, Thread},
    Mentions,
};

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

    /// A constructor to create a plain text message.
    pub fn text_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::text_plain(body))
    }

    /// A constructor to create an html message.
    pub fn text_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::text_html(body, html_body))
    }

    /// A constructor to create a markdown message.
    #[cfg(feature = "markdown")]
    pub fn text_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::new(MessageType::text_markdown(body))
    }

    /// A constructor to create a plain text notice.
    pub fn notice_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::notice_plain(body))
    }

    /// A constructor to create an html notice.
    pub fn notice_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::notice_html(body, html_body))
    }

    /// A constructor to create a markdown notice.
    #[cfg(feature = "markdown")]
    pub fn notice_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::new(MessageType::notice_markdown(body))
    }

    /// A constructor to create a plain text emote.
    pub fn emote_plain(body: impl Into<String>) -> Self {
        Self::new(MessageType::emote_plain(body))
    }

    /// A constructor to create an html emote.
    pub fn emote_html(body: impl Into<String>, html_body: impl Into<String>) -> Self {
        Self::new(MessageType::emote_html(body, html_body))
    }

    /// A constructor to create a markdown emote.
    #[cfg(feature = "markdown")]
    pub fn emote_markdown(body: impl AsRef<str> + Into<String>) -> Self {
        Self::new(MessageType::emote_markdown(body))
    }

    /// Transform `self` into a `RoomMessageEventContent` with the given relation.
    pub fn with_relation(
        self,
        relates_to: Option<Relation<RoomMessageEventContentWithoutRelation>>,
    ) -> RoomMessageEventContent {
        let Self { msgtype, mentions } = self;
        RoomMessageEventContent { msgtype, relates_to, mentions }
    }

    /// Turns `self` into a [rich reply] to the message using the given metadata.
    ///
    /// Sets the `in_reply_to` field inside `relates_to`, and optionally the `rel_type` to
    /// `m.thread` if the metadata has a `thread` and `ForwardThread::Yes` is used.
    ///
    /// If `AddMentions::Yes` is used, the `sender` in the metadata is added as a user mention.
    ///
    /// [rich reply]: https://spec.matrix.org/latest/client-server-api/#rich-replies
    #[track_caller]
    pub fn make_reply_to<'a>(
        mut self,
        metadata: impl Into<ReplyMetadata<'a>>,
        forward_thread: ForwardThread,
        add_mentions: AddMentions,
    ) -> RoomMessageEventContent {
        let metadata = metadata.into();
        let original_event_id = metadata.event_id.to_owned();

        let original_thread_id = metadata
            .thread
            .filter(|_| forward_thread == ForwardThread::Yes)
            .map(|thread| thread.event_id.clone());
        let relates_to = if let Some(event_id) = original_thread_id {
            Relation::Thread(Thread::plain(event_id.to_owned(), original_event_id.to_owned()))
        } else {
            Relation::Reply { in_reply_to: InReplyTo { event_id: original_event_id.to_owned() } }
        };

        if add_mentions == AddMentions::Yes {
            self.mentions
                .get_or_insert_with(Mentions::new)
                .user_ids
                .insert(metadata.sender.to_owned());
        }

        self.with_relation(Some(relates_to))
    }

    /// Turns `self` into a new message for a [thread], that is optionally a reply.
    ///
    /// Looks for the `thread` in the given metadata. If it exists, this message will be in the same
    /// thread. If it doesn't, a new thread is created with the `event_id` in the metadata as the
    /// root.
    ///
    /// It also sets the `in_reply_to` field inside `relates_to` to point the `event_id`
    /// in the metadata. If `ReplyWithinThread::Yes` is used, the metadata should be constructed
    /// from the event to make a reply to, otherwise it should be constructed from the latest
    /// event in the thread.
    ///
    /// If `AddMentions::Yes` is used, the `sender` in the metadata is added as a user mention.
    ///
    /// [thread]: https://spec.matrix.org/latest/client-server-api/#threading
    pub fn make_for_thread<'a>(
        self,
        metadata: impl Into<ReplyMetadata<'a>>,
        is_reply: ReplyWithinThread,
        add_mentions: AddMentions,
    ) -> RoomMessageEventContent {
        let metadata = metadata.into();

        let mut content = if is_reply == ReplyWithinThread::Yes {
            self.make_reply_to(metadata, ForwardThread::No, add_mentions)
        } else {
            self.into()
        };

        let thread_root = if let Some(Thread { event_id, .. }) = &metadata.thread {
            event_id.to_owned()
        } else {
            metadata.event_id.to_owned()
        };

        content.relates_to = Some(Relation::Thread(Thread {
            event_id: thread_root,
            in_reply_to: Some(InReplyTo { event_id: metadata.event_id.to_owned() }),
            is_falling_back: is_reply == ReplyWithinThread::No,
        }));

        content
    }

    /// Turns `self` into a [replacement] (or edit) for a given message.
    ///
    /// The first argument after `self` can be `&OriginalRoomMessageEvent` or
    /// `&OriginalSyncRoomMessageEvent` if you don't want to create `ReplacementMetadata` separately
    /// before calling this function.
    ///
    /// This takes the content and sets it in `m.new_content`, and modifies the `content` to include
    /// a fallback.
    ///
    /// If this message contains [`Mentions`], they are copied into `m.new_content` to keep the same
    /// mentions, but the ones in `content` are filtered with the ones in the
    /// [`ReplacementMetadata`] so only new mentions will trigger a notification.
    ///
    /// # Panics
    ///
    /// Panics if `self` has a `formatted_body` with a format other than HTML.
    ///
    /// [replacement]: https://spec.matrix.org/latest/client-server-api/#event-replacements
    #[track_caller]
    pub fn make_replacement(
        mut self,
        metadata: impl Into<ReplacementMetadata>,
    ) -> RoomMessageEventContent {
        let metadata = metadata.into();

        let mentions = self.mentions.take();

        // Only set mentions that were not there before.
        if let Some(mentions) = &mentions {
            let new_mentions = metadata.mentions.map(|old_mentions| {
                let mut new_mentions = Mentions::new();

                new_mentions.user_ids = mentions
                    .user_ids
                    .iter()
                    .filter(|u| !old_mentions.user_ids.contains(*u))
                    .cloned()
                    .collect();

                new_mentions.room = mentions.room && !old_mentions.room;

                new_mentions
            });

            self.mentions = new_mentions;
        };

        // Prepare relates_to with the untouched msgtype.
        let relates_to = Relation::Replacement(Replacement {
            event_id: metadata.event_id,
            new_content: RoomMessageEventContentWithoutRelation {
                msgtype: self.msgtype.clone(),
                mentions,
            },
        });

        self.msgtype.make_replacement_body();

        let mut content = RoomMessageEventContent::from(self);
        content.relates_to = Some(relates_to);

        content
    }

    /// Add the given [mentions] to this event.
    ///
    /// If no [`Mentions`] was set on this events, this sets it. Otherwise, this updates the current
    /// mentions by extending the previous `user_ids` with the new ones, and applies a logical OR to
    /// the values of `room`.
    ///
    /// [mentions]: https://spec.matrix.org/latest/client-server-api/#user-and-room-mentions
    pub fn add_mentions(mut self, mentions: Mentions) -> Self {
        self.mentions.get_or_insert_with(Mentions::new).add(mentions);
        self
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

impl From<RoomMessageEventContentWithoutRelation> for RoomMessageEventContent {
    fn from(value: RoomMessageEventContentWithoutRelation) -> Self {
        let RoomMessageEventContentWithoutRelation { msgtype, mentions } = value;
        Self { msgtype, relates_to: None, mentions }
    }
}
