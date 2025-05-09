//! Types for the [`m.media_preview_config`] event.
//!
//! [`m.media_preview_config`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4278

use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize};

use crate::{macros::EventContent, PrivOwnedStr};

/// The content of an `m.media_preview_config` event.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.media_preview_config", kind = GlobalAccountData)]
pub struct MediaPreviewConfigEventContent {
    /// The media previews configuration.
    #[serde(default)]
    pub media_previews: MediaPreviews,

    /// The invite avatars configuration.
    #[serde(default)]
    pub invite_avatars: InviteAvatars,
}

/// The content of an `io.element.msc4278.media_preview_config` event,
/// the unstable version of `m.media_preview_config` in global account data.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "io.element.msc4278.media_preview_config", kind = GlobalAccountData)]
#[serde(transparent)]
pub struct UnstableMediaPreviewConfigEventContent(pub MediaPreviewConfigEventContent);

/// The configuration that handles if media previews should be shown in the timeline.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum, Default)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MediaPreviews {
    /// Media previews should be hidden.
    Off,

    /// Media previews should be only shown in private rooms.
    Private,

    /// Media previews should always be shown.
    #[default]
    On,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The configuration to handle if avatars should be shown in invites.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum, Default)]
#[ruma_enum(rename_all = "lowercase")]
#[non_exhaustive]
pub enum InviteAvatars {
    /// Avatars in invites should be hidden.
    Off,

    /// Avatars in invites should be shown.
    #[default]
    On,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl MediaPreviewConfigEventContent {
    /// Create a new [`MediaPreviewConfigEventContent`] with the given values.
    pub fn new(media_previews: MediaPreviews, invite_avatars: InviteAvatars) -> Self {
        Self { media_previews, invite_avatars }
    }
}

impl std::ops::Deref for UnstableMediaPreviewConfigEventContent {
    type Target = MediaPreviewConfigEventContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MediaPreviewConfigEventContent> for UnstableMediaPreviewConfigEventContent {
    fn from(value: MediaPreviewConfigEventContent) -> Self {
        Self(value)
    }
}

impl From<UnstableMediaPreviewConfigEventContent> for MediaPreviewConfigEventContent {
    fn from(value: UnstableMediaPreviewConfigEventContent) -> Self {
        value.0
    }
}

#[cfg(all(test, feature = "unstable-msc4278"))]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{MediaPreviewConfigEventContent, UnstableMediaPreviewConfigEventContent};
    use crate::{
        media_preview_config::{InviteAvatars, MediaPreviews},
        AnyGlobalAccountDataEvent, GlobalAccountDataEvent,
    };

    #[test]
    fn deserialize() {
        let raw_unstable_media_preview_config = json!({
            "type": "io.element.msc4278.media_preview_config",
            "content": {
                "media_previews": "private",
                "invite_avatars": "off",
            },
        });
        let unstable_media_preview_config_data =
            from_json_value::<AnyGlobalAccountDataEvent>(raw_unstable_media_preview_config)
                .unwrap();
        assert_matches!(
            unstable_media_preview_config_data,
            AnyGlobalAccountDataEvent::UnstableMediaPreviewConfig(unstable_media_preview_config)
        );
        assert_eq!(unstable_media_preview_config.content.media_previews, MediaPreviews::Private);
        assert_eq!(unstable_media_preview_config.content.invite_avatars, InviteAvatars::Off);

        let raw_media_preview_config = json!({
            "type": "m.media_preview_config",
            "content": {
                "media_previews": "on",
                "invite_avatars": "on",
            },
        });
        let media_preview_config_data =
            from_json_value::<AnyGlobalAccountDataEvent>(raw_media_preview_config).unwrap();
        assert_matches!(
            media_preview_config_data,
            AnyGlobalAccountDataEvent::MediaPreviewConfig(media_preview_config)
        );
        assert_eq!(media_preview_config.content.media_previews, MediaPreviews::On);
        assert_eq!(media_preview_config.content.invite_avatars, InviteAvatars::On);
    }

    #[test]
    fn serialize() {
        let unstable_media_preview_config = UnstableMediaPreviewConfigEventContent(
            MediaPreviewConfigEventContent::new(MediaPreviews::Off, InviteAvatars::On),
        );
        let unstable_media_preview_config_account_data =
            GlobalAccountDataEvent { content: unstable_media_preview_config };
        assert_eq!(
            to_json_value(unstable_media_preview_config_account_data).unwrap(),
            json!({
                "type": "io.element.msc4278.media_preview_config",
                "content": {
                    "media_previews": "off",
                    "invite_avatars": "on",
                },
            })
        );

        let media_preview_config =
            MediaPreviewConfigEventContent::new(MediaPreviews::On, InviteAvatars::Off);
        let media_preview_config_account_data =
            GlobalAccountDataEvent { content: media_preview_config };
        assert_eq!(
            to_json_value(media_preview_config_account_data).unwrap(),
            json!({
                "type": "m.media_preview_config",
                "content": {
                    "media_previews": "on",
                    "invite_avatars": "off",
                },
            })
        );
    }
}
