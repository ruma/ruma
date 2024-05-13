use as_variant::as_variant;
use ruma_common::{serde::Raw, OwnedEventId, OwnedUserId, RoomId, UserId};
use serde::{Deserialize, Serialize};

use super::{
    AddMentions, ForwardThread, MessageType, OriginalRoomMessageEvent, Relation,
    ReplacementMetadata, ReplyWithinThread, RoomMessageEventContent,
};
use crate::{
    relation::{InReplyTo, Replacement, Thread},
    room::message::{reply::OriginalEventData, FormattedBody},
    AnySyncTimelineEvent, Mentions,
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

    /// Turns `self` into a reply to the given message.
    ///
    /// Takes the `body` / `formatted_body` (if any) in `self` for the main text and prepends a
    /// quoted version of `original_message`. Also sets the `in_reply_to` field inside `relates_to`,
    /// and optionally the `rel_type` to `m.thread` if the `original_message is in a thread and
    /// thread forwarding is enabled.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
    ///
    /// # Panics
    ///
    /// Panics if `self` has a `formatted_body` with a format other than HTML.
    #[track_caller]
    pub fn make_reply_to(
        mut self,
        original_message: &OriginalRoomMessageEvent,
        forward_thread: ForwardThread,
        add_mentions: AddMentions,
    ) -> RoomMessageEventContent {
        self.msgtype.add_reply_fallback(original_message.into());
        let original_event_id = original_message.event_id.clone();

        let original_thread_id = if forward_thread == ForwardThread::Yes {
            original_message
                .content
                .relates_to
                .as_ref()
                .and_then(as_variant!(Relation::Thread))
                .map(|thread| thread.event_id.clone())
        } else {
            None
        };

        let sender_for_mentions =
            (add_mentions == AddMentions::Yes).then_some(&*original_message.sender);

        self.make_reply_tweaks(original_event_id, original_thread_id, sender_for_mentions)
    }

    /// Turns `self` into a reply to the given raw event.
    ///
    /// Takes the `body` / `formatted_body` (if any) in `self` for the main text and prepends a
    /// quoted version of the `body` of `original_event` (if any). Also sets the `in_reply_to` field
    /// inside `relates_to`, and optionally the `rel_type` to `m.thread` if the
    /// `original_message is in a thread and thread forwarding is enabled.
    ///
    /// It is recommended to use [`Self::make_reply_to()`] for replies to `m.room.message` events,
    /// as the generated fallback is better for some `msgtype`s.
    ///
    /// Note that except for the panic below, this is infallible. Which means that if a field is
    /// missing when deserializing the data, the changes that require it will not be applied. It
    /// will still at least apply the `m.in_reply_to` relation to this content.
    ///
    /// # Panics
    ///
    /// Panics if `self` has a `formatted_body` with a format other than HTML.
    #[track_caller]
    pub fn make_reply_to_raw(
        mut self,
        original_event: &Raw<AnySyncTimelineEvent>,
        original_event_id: OwnedEventId,
        room_id: &RoomId,
        forward_thread: ForwardThread,
        add_mentions: AddMentions,
    ) -> RoomMessageEventContent {
        #[derive(Deserialize)]
        struct ContentDeHelper {
            body: Option<String>,
            #[serde(flatten)]
            formatted: Option<FormattedBody>,
            #[cfg(feature = "unstable-msc1767")]
            #[serde(rename = "org.matrix.msc1767.text")]
            text: Option<String>,
            #[serde(rename = "m.relates_to")]
            relates_to: Option<crate::room::encrypted::Relation>,
        }

        let sender = original_event.get_field::<OwnedUserId>("sender").ok().flatten();
        let content = original_event.get_field::<ContentDeHelper>("content").ok().flatten();
        let relates_to = content.as_ref().and_then(|c| c.relates_to.as_ref());

        let content_body = content.as_ref().and_then(|c| {
            let body = c.body.as_deref();
            #[cfg(feature = "unstable-msc1767")]
            let body = body.or(c.text.as_deref());

            Some((c, body?))
        });

        // Only apply fallback if we managed to deserialize raw event.
        if let (Some(sender), Some((content, body))) = (&sender, content_body) {
            let is_reply =
                matches!(content.relates_to, Some(crate::room::encrypted::Relation::Reply { .. }));
            let data = OriginalEventData {
                body,
                formatted: content.formatted.as_ref(),
                is_emote: false,
                is_reply,
                room_id,
                event_id: &original_event_id,
                sender,
            };

            self.msgtype.add_reply_fallback(data);
        }

        let original_thread_id = if forward_thread == ForwardThread::Yes {
            relates_to
                .and_then(as_variant!(crate::room::encrypted::Relation::Thread))
                .map(|thread| thread.event_id.clone())
        } else {
            None
        };

        let sender_for_mentions = sender.as_deref().filter(|_| add_mentions == AddMentions::Yes);
        self.make_reply_tweaks(original_event_id, original_thread_id, sender_for_mentions)
    }

    /// Turns `self` into a new message for a thread, that is optionally a reply.
    ///
    /// Looks for a [`Relation::Thread`] in `previous_message`. If it exists, this message will be
    /// in the same thread. If it doesn't, a new thread with `previous_message` as the root is
    /// created.
    ///
    /// If this is a reply within the thread, takes the `body` / `formatted_body` (if any) in `self`
    /// for the main text and prepends a quoted version of `previous_message`. Also sets the
    /// `in_reply_to` field inside `relates_to`.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
    ///
    /// # Panics
    ///
    /// Panics if this is a reply within the thread and `self` has a `formatted_body` with a format
    /// other than HTML.
    pub fn make_for_thread(
        self,
        previous_message: &OriginalRoomMessageEvent,
        is_reply: ReplyWithinThread,
        add_mentions: AddMentions,
    ) -> RoomMessageEventContent {
        let mut content = if is_reply == ReplyWithinThread::Yes {
            self.make_reply_to(previous_message, ForwardThread::No, add_mentions)
        } else {
            self.into()
        };

        let thread_root = if let Some(Relation::Thread(Thread { event_id, .. })) =
            &previous_message.content.relates_to
        {
            event_id.clone()
        } else {
            previous_message.event_id.clone()
        };

        content.relates_to = Some(Relation::Thread(Thread {
            event_id: thread_root,
            in_reply_to: Some(InReplyTo { event_id: previous_message.event_id.clone() }),
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
    /// If the message that is replaced is a reply to another message, the latter should also be
    /// provided to be able to generate a rich reply fallback that takes the `body` /
    /// `formatted_body` (if any) in `self` for the main text and prepends a quoted version of
    /// `original_message`.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/rich_reply.md"))]
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
        replied_to_message: Option<&OriginalRoomMessageEvent>,
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

        // Add reply fallback if needed.
        let mut content = if let Some(original_message) = replied_to_message {
            self.make_reply_to(original_message, ForwardThread::No, AddMentions::No)
        } else {
            self.into()
        };

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

    fn make_reply_tweaks(
        mut self,
        original_event_id: OwnedEventId,
        original_thread_id: Option<OwnedEventId>,
        sender_for_mentions: Option<&UserId>,
    ) -> RoomMessageEventContent {
        let relates_to = if let Some(event_id) = original_thread_id {
            Relation::Thread(Thread::plain(event_id.to_owned(), original_event_id.to_owned()))
        } else {
            Relation::Reply { in_reply_to: InReplyTo { event_id: original_event_id.to_owned() } }
        };

        if let Some(sender) = sender_for_mentions {
            self.mentions.get_or_insert_with(Mentions::new).user_ids.insert(sender.to_owned());
        }

        self.with_relation(Some(relates_to))
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
