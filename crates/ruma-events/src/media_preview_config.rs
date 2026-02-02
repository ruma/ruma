//! Types for the [`m.media_preview_config`] event.
//!
//! [`m.media_preview_config`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4278

use ruma_common::serde::JsonCastable;
use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize};

use crate::{PrivOwnedStr, macros::EventContent};

/// The content of an `m.media_preview_config` event.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.media_preview_config", kind = GlobalAccountData + RoomAccountData)]
pub struct MediaPreviewConfigEventContent {
    /// The media previews configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_previews: Option<MediaPreviews>,

    /// The invite avatars configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_avatars: Option<InviteAvatars>,
}

impl JsonCastable<UnstableMediaPreviewConfigEventContent> for MediaPreviewConfigEventContent {}

/// The content of an `io.element.msc4278.media_preview_config` event,
/// the unstable version of `m.media_preview_config` in global account data.
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "io.element.msc4278.media_preview_config", kind = GlobalAccountData + RoomAccountData)]
#[serde(transparent)]
pub struct UnstableMediaPreviewConfigEventContent(pub MediaPreviewConfigEventContent);

impl JsonCastable<MediaPreviewConfigEventContent> for UnstableMediaPreviewConfigEventContent {}

/// The configuration that handles if media previews should be shown in the timeline.
#[derive(Clone, StringEnum, Default)]
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
#[derive(Clone, StringEnum, Default)]
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
    /// Create a new empty [`MediaPreviewConfigEventContent`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the value of the setting for the media previews.
    pub fn media_previews(mut self, media_previews: Option<MediaPreviews>) -> Self {
        self.media_previews = media_previews;
        self
    }

    /// Set the value of the setting for the media previews.
    pub fn invite_avatars(mut self, invite_avatars: Option<InviteAvatars>) -> Self {
        self.invite_avatars = invite_avatars;
        self
    }

    /// Merge the config from the global account data with the config from the room account data.
    ///
    /// The values that are set in the room account data take precedence over the values in the
    /// global account data.
    pub fn merge_global_and_room_config(global_config: Self, room_config: Self) -> Self {
        Self {
            media_previews: room_config.media_previews.or(global_config.media_previews),
            invite_avatars: room_config.invite_avatars.or(global_config.invite_avatars),
        }
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
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    use serde_json::{from_value as from_json_value, json};

    use super::{MediaPreviewConfigEventContent, UnstableMediaPreviewConfigEventContent};
    use crate::{
        AnyGlobalAccountDataEvent, GlobalAccountDataEvent,
        media_preview_config::{InviteAvatars, MediaPreviews},
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
        assert_eq!(
            unstable_media_preview_config.content.media_previews,
            Some(MediaPreviews::Private)
        );
        assert_eq!(unstable_media_preview_config.content.invite_avatars, Some(InviteAvatars::Off));

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
        assert_eq!(media_preview_config.content.media_previews, Some(MediaPreviews::On));
        assert_eq!(media_preview_config.content.invite_avatars, Some(InviteAvatars::On));
    }

    #[test]
    fn serialize() {
        let unstable_media_preview_config = UnstableMediaPreviewConfigEventContent(
            MediaPreviewConfigEventContent::new()
                .media_previews(Some(MediaPreviews::Off))
                .invite_avatars(Some(InviteAvatars::On)),
        );
        let unstable_media_preview_config_account_data =
            GlobalAccountDataEvent { content: unstable_media_preview_config };
        assert_to_canonical_json_eq!(
            unstable_media_preview_config_account_data,
            json!({
                "type": "io.element.msc4278.media_preview_config",
                "content": {
                    "media_previews": "off",
                    "invite_avatars": "on",
                },
            })
        );

        let media_preview_config = MediaPreviewConfigEventContent::new()
            .media_previews(Some(MediaPreviews::On))
            .invite_avatars(Some(InviteAvatars::Off));
        let media_preview_config_account_data =
            GlobalAccountDataEvent { content: media_preview_config };
        assert_to_canonical_json_eq!(
            media_preview_config_account_data,
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
