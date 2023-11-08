<!-- Keep this comment so the content is always included as a new paragraph -->
This function requires an [`OriginalRoomMessageEvent`] since it creates a permalink to
the previous message, for which the room ID is required. If you want to reply to an
[`OriginalSyncRoomMessageEvent`], you have to convert it first by calling [`.into_full_event()`].

If the message was edited, the previous message should be the original message that was edited,
with the content of its replacement, to allow the fallback to be accurate at the time it is added.

It is recommended to enable the `html` feature when using this method as this will
clean up nested [rich reply fallbacks] in chains of replies. This uses [`sanitize_html()`]
internally, with [`RemoveReplyFallback::Yes`].

[`OriginalRoomMessageEvent`]: crate::room::message::OriginalRoomMessageEvent
[`OriginalSyncRoomMessageEvent`]: crate::room::message::OriginalSyncRoomMessageEvent
[`.into_full_event()`]: crate::OriginalSyncMessageLikeEvent::into_full_event
[rich reply fallbacks]: https://spec.matrix.org/latest/client-server-api/#fallbacks-for-rich-replies
[`sanitize_html()`]: ruma_html::sanitize_html
[`RemoveReplyFallback::Yes`]: ruma_html::RemoveReplyFallback::Yes
