//! Types for the [`m.recent_emoji`] account data event.
//!
//! [`m.recent_emoji`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4356

use js_int::{UInt, uint};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

/// The content of an [`m.recent_emoji`] event.
///
/// [`m.recent_emoji`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4356
#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.recent_emoji", kind = GlobalAccountData)]
pub struct RecentEmojiEventContent {
    /// The list of recently used emojis, ordered by last usage time.
    #[serde(default, deserialize_with = "ruma_common::serde::ignore_invalid_vec_items")]
    pub recent_emoji: Vec<RecentEmoji>,
}

impl RecentEmojiEventContent {
    /// The maximum length of the list recommended in the Matrix specification.
    pub const RECOMMENDED_MAX_LEN: usize = 100;

    /// Creates a new `RecentEmojiEventContent` from the given list.
    pub fn new(recent_emoji: Vec<RecentEmoji>) -> Self {
        Self { recent_emoji }
    }

    /// Increment the total for the given emoji.
    ///
    /// If the emoji is in the list, its total is incremented and it is moved to the start of the
    /// list.
    ///
    /// If the emoji is not in the list, it is added at the start of the list with a total set to
    /// `1`.
    ///
    /// If the length of the list is bigger than [`RECOMMENDED_MAX_LEN`](Self::RECOMMENDED_MAX_LEN),
    /// the list is truncated.
    pub fn increment_emoji_total(&mut self, emoji: &str) {
        // Start by truncating the list if necessary to make sure that shifting items doesn't take
        // too much time.
        self.recent_emoji.truncate(Self::RECOMMENDED_MAX_LEN);

        if let Some(position) = self.recent_emoji.iter().position(|e| e.emoji == emoji) {
            let total = &mut self.recent_emoji[position].total;
            *total = (*total).saturating_add(uint!(1));

            if position > 0 {
                let emoji = self.recent_emoji.remove(position);
                self.recent_emoji.insert(0, emoji);
            }
        } else {
            let emoji = RecentEmoji::new(emoji.to_owned());
            self.recent_emoji.insert(0, emoji);

            // Truncate it again if necessary.
            self.recent_emoji.truncate(Self::RECOMMENDED_MAX_LEN);
        }
    }

    /// Get the list of recent emoji sorted by the number of uses.
    ///
    /// When several emoji have the same number of uses they are sorted by last usage time.
    ///
    /// The returned list is truncated to [`RECOMMENDED_MAX_LEN`](Self::RECOMMENDED_MAX_LEN).
    pub fn recent_emoji_sorted_by_total(&self) -> Vec<RecentEmoji> {
        let mut recent_emoji =
            self.recent_emoji.iter().take(Self::RECOMMENDED_MAX_LEN).cloned().collect::<Vec<_>>();
        // We reverse the sorting to get the highest count first.
        recent_emoji.sort_by(|lhs, rhs| rhs.total.cmp(&lhs.total));
        recent_emoji
    }
}

/// A recently used emoji.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct RecentEmoji {
    /// The emoji as a string.
    pub emoji: String,

    /// The number of times the emoji has been used.
    pub total: UInt,
}

impl RecentEmoji {
    /// Creates a new `RecentEmoji` for the given emoji.
    ///
    /// The total is set to `1`.
    pub fn new(emoji: String) -> Self {
        Self { emoji, total: uint!(1) }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use js_int::uint;
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::{RecentEmoji, RecentEmojiEventContent};
    use crate::AnyGlobalAccountDataEvent;

    #[test]
    fn recent_emoji_serialization() {
        let content = RecentEmojiEventContent::new([RecentEmoji::new("😎".to_owned())].into());

        assert_to_canonical_json_eq!(
            content,
            json!({
                "recent_emoji": [{
                    "emoji": "😎",
                    "total": 1,
                }],
            }),
        );
    }

    #[test]
    fn recent_emoji_deserialization() {
        let json = json!({
            "content": {
                "recent_emoji": [
                    {
                        "emoji": "😎",
                        "total": 1,
                    },
                    // Invalid item that will be ignored.
                    {
                        "emoji": "🏠",
                        "total": -1,
                    },
                ],
            },
            "type": "m.recent_emoji",
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json),
            Ok(AnyGlobalAccountDataEvent::RecentEmoji(ev))
        );
        assert_eq!(ev.content.recent_emoji, [RecentEmoji::new("😎".to_owned())]);
    }

    #[test]
    fn recent_emoji_increment() {
        let json = json!({
            "recent_emoji": [
                {
                    "emoji": "😎",
                    "total": 1,
                },
                {
                    "emoji": "🏠",
                    "total": 5,
                },
                {
                    "emoji": "🧑‍💻",
                    "total": 2,
                },
            ],
        });
        let mut content = from_json_value::<RecentEmojiEventContent>(json).unwrap();

        // Check first the initial order.
        let mut iter = content.recent_emoji.iter();
        assert_eq!(iter.next().unwrap().emoji, "😎");
        assert_eq!(iter.next().unwrap().emoji, "🏠");
        assert_eq!(iter.next().unwrap().emoji, "🧑‍💻");
        assert_eq!(iter.next(), None);

        // Increment a known emoji.
        content.increment_emoji_total("🏠");
        assert_eq!(content.recent_emoji.first().unwrap().total, uint!(6));

        let mut iter = content.recent_emoji.iter();
        assert_eq!(iter.next().unwrap().emoji, "🏠");
        assert_eq!(iter.next().unwrap().emoji, "😎");
        assert_eq!(iter.next().unwrap().emoji, "🧑‍💻");
        assert_eq!(iter.next(), None);

        // Increment an unknown emoji.
        content.increment_emoji_total("💩");
        assert_eq!(content.recent_emoji.first().unwrap().total, uint!(1));

        let mut iter = content.recent_emoji.iter();
        assert_eq!(iter.next().unwrap().emoji, "💩");
        assert_eq!(iter.next().unwrap().emoji, "🏠");
        assert_eq!(iter.next().unwrap().emoji, "😎");
        assert_eq!(iter.next().unwrap().emoji, "🧑‍💻");
        assert_eq!(iter.next(), None);

        // Construct a list of more than 100 emojis.
        let first_emoji = "\u{2700}";
        let first_emoji_u32 = 0x2700_u32;
        let mut content = RecentEmojiEventContent::new(
            std::iter::repeat_n(first_emoji_u32, 110)
                .enumerate()
                .map(|(n, start)| {
                    let char = char::from_u32(start + (n as u32)).unwrap();
                    RecentEmoji::new(char.into())
                })
                .collect(),
        );
        assert_eq!(content.recent_emoji.len(), 110);

        // Increment the first emoji, the list should be truncated.
        content.increment_emoji_total(first_emoji);
        assert_eq!(content.recent_emoji.first().unwrap().total, uint!(2));
        assert_eq!(content.recent_emoji.len(), 100);
    }

    #[test]
    fn recent_emoji_sorted_by_total() {
        let json = json!({
            "recent_emoji": [
                {
                    "emoji": "😎",
                    "total": 1,
                },
                {
                    "emoji": "🏠",
                    "total": 5,
                },
                {
                    "emoji": "🧑‍💻",
                    "total": 2,
                },
                {
                    "emoji": "🚀",
                    "total": 1,
                },
            ],
        });
        let content = from_json_value::<RecentEmojiEventContent>(json).unwrap();

        // Check first the initial order.
        let mut iter = content.recent_emoji.iter();
        assert_eq!(iter.next().unwrap().emoji, "😎");
        assert_eq!(iter.next().unwrap().emoji, "🏠");
        assert_eq!(iter.next().unwrap().emoji, "🧑‍💻");
        assert_eq!(iter.next().unwrap().emoji, "🚀");
        assert_eq!(iter.next(), None);

        // Check the sorted order.
        let sorted = content.recent_emoji_sorted_by_total();
        let mut iter = sorted.iter();
        assert_eq!(iter.next().unwrap().emoji, "🏠");
        assert_eq!(iter.next().unwrap().emoji, "🧑‍💻");
        assert_eq!(iter.next().unwrap().emoji, "😎");
        assert_eq!(iter.next().unwrap().emoji, "🚀");
        assert_eq!(iter.next(), None);
    }
}
