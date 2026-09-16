//! A presence event is represented by a struct with a set content field.
//!
//! The only content valid for this event is `PresenceEventContent`.

#[cfg(feature = "unstable-msc4532")]
pub mod persistent;
#[cfg(feature = "unstable-msc4495")]
pub mod prompted;
#[cfg(feature = "unstable-msc4495")]
pub mod sharing;

use js_int::UInt;
#[cfg(feature = "unstable-msc4532")]
use ruma_common::presence::PresenceStatus;
use ruma_common::{OwnedMxcUri, OwnedUserId, presence::PresenceState};
use serde::{Deserialize, Serialize};

/// Presence event.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::exhaustive_structs)]
#[serde(tag = "type", rename = "m.presence")]
pub struct PresenceEvent {
    /// Data specific to the event type.
    pub content: PresenceEventContent,

    /// Contains the fully-qualified ID of the user who sent this event.
    pub sender: OwnedUserId,
}

/// The over-the-wire format for [`PresenceEventContent`]. This exists to enable a custom
/// (de)serialization implementation providing backwards-compatibility for the `status_msg`
/// field when [MSC4532] is enabled.
///
/// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
struct PresenceEventRepr {
    /// The current avatar URL for this user.
    ///
    /// If you activate the `compat-empty-string-null` feature, this field being an empty string in
    /// JSON will result in `None` here during deserialization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "compat-empty-string-null",
        serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
    )]
    pub avatar_url: Option<OwnedMxcUri>,

    /// Whether or not the user is currently active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currently_active: Option<bool>,

    /// The current display name for this user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayname: Option<String>,

    /// The last time since this user performed some action, in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_ago: Option<UInt>,

    /// The presence state for this user.
    pub presence: PresenceState,

    /// An optional description to accompany the presence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_msg: Option<String>,

    /// Optional information to accompany the presence.
    ///
    /// This field uses the unstable prefix defined in [MSC4532].
    ///
    /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
    #[cfg(feature = "unstable-msc4532")]
    #[serde(
        skip_serializing_if = "ruma_common::serde::is_default",
        rename = "org.continuwuity.presence_v2.msc4532.status",
        default
    )]
    pub status: PresenceStatus,
}

/// Informs the room of members presence.
///
/// This is the only type a `PresenceEvent` can contain as its `content` field.
#[derive(Clone, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct PresenceEventContent {
    /// The current avatar URL for this user.
    ///
    /// If you activate the `compat-empty-string-null` feature, this field being an empty string in
    /// JSON will result in `None` here during deserialization.
    pub avatar_url: Option<OwnedMxcUri>,

    /// Whether or not the user is currently active.
    pub currently_active: Option<bool>,

    /// The current display name for this user.
    pub displayname: Option<String>,

    /// The last time since this user performed some action, in milliseconds.
    pub last_active_ago: Option<UInt>,

    /// The presence state for this user.
    pub presence: PresenceState,

    /// An optional description to accompany the presence.
    #[cfg(not(feature = "unstable-msc4532"))]
    pub status_msg: Option<String>,

    /// Optional information to accompany the presence.
    ///
    /// This field uses the unstable prefix defined in [MSC4532].
    ///
    /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
    #[cfg(feature = "unstable-msc4532")]
    pub status: PresenceStatus,
}

impl<'de> Deserialize<'de> for PresenceEventContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let repr = PresenceEventRepr::deserialize(deserializer)?;
        Ok(Self {
            avatar_url: repr.avatar_url,
            currently_active: repr.currently_active,
            displayname: repr.displayname,
            last_active_ago: repr.last_active_ago,
            presence: repr.presence,
            #[cfg(not(feature = "unstable-msc4532"))]
            status_msg: repr.status_msg,
            #[cfg(feature = "unstable-msc4532")]
            status: if repr.status == PresenceStatus::default() {
                PresenceStatus::new(repr.status_msg)
            } else {
                repr.status
            },
        })
    }
}
impl Serialize for PresenceEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        PresenceEventRepr {
            avatar_url: self.avatar_url.clone(),
            currently_active: self.currently_active,
            displayname: self.displayname.clone(),
            last_active_ago: self.last_active_ago,
            presence: self.presence.clone(),
            #[cfg(not(feature = "unstable-msc4532"))]
            status_msg: self.status_msg.clone(),
            #[cfg(feature = "unstable-msc4532")]
            status_msg: self.status.msg.clone(),
            #[cfg(feature = "unstable-msc4532")]
            status: self.status.clone(),
        }
        .serialize(serializer)
    }
}

impl PresenceEventContent {
    /// Creates a new `PresenceEventContent` with the given state.
    pub fn new(presence: PresenceState) -> Self {
        Self {
            avatar_url: None,
            currently_active: None,
            displayname: None,
            last_active_ago: None,
            presence,
            #[cfg(not(feature = "unstable-msc4532"))]
            status_msg: None,
            #[cfg(feature = "unstable-msc4532")]
            status: PresenceStatus::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    #[cfg(feature = "unstable-msc4532")]
    use ruma_common::presence::PresenceStatus;
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, mxc_uri, owned_mxc_uri,
        presence::PresenceState,
    };
    use serde_json::{from_value as from_json_value, json};

    use super::{PresenceEvent, PresenceEventContent};

    #[test]
    fn serialization() {
        let content = PresenceEventContent {
            avatar_url: Some(owned_mxc_uri!("mxc://localhost/wefuiwegh8742w")),
            currently_active: Some(false),
            displayname: None,
            last_active_ago: Some(uint!(2_478_593)),
            presence: PresenceState::Online,
            #[cfg(not(feature = "unstable-msc4532"))]
            status_msg: Some("Making cupcakes".into()),
            #[cfg(feature = "unstable-msc4532")]
            status: PresenceStatus::new(Some("Making cupcakes".into())),
        };

        #[cfg(feature = "unstable-msc4532")]
        assert_to_canonical_json_eq!(
            content,
            json!({
                "avatar_url": "mxc://localhost/wefuiwegh8742w",
                "currently_active": false,
                "last_active_ago": 2_478_593,
                "presence": "online",
                "org.continuwuity.presence_v2.msc4532.status": {
                    "msg": "Making cupcakes"
                },
                "status_msg": "Making cupcakes"
            }),
        );
        #[cfg(not(feature = "unstable-msc4532"))]
        assert_to_canonical_json_eq!(
            content,
            json!({
                "avatar_url": "mxc://localhost/wefuiwegh8742w",
                "currently_active": false,
                "last_active_ago": 2_478_593,
                "presence": "online",
                "status_msg": "Making cupcakes",
            }),
        );
    }

    #[test]
    fn deserialization() {
        let json = json!({
            "content": {
                "avatar_url": "mxc://localhost/wefuiwegh8742w",
                "currently_active": false,
                "last_active_ago": 2_478_593,
                "presence": "online",
                "status_msg": "Making cupcakes"
            },
            "sender": "@example:localhost",
            "type": "m.presence"
        });

        let ev = from_json_value::<PresenceEvent>(json).unwrap();
        assert_eq!(
            ev.content.avatar_url.as_deref(),
            Some(mxc_uri!("mxc://localhost/wefuiwegh8742w"))
        );
        assert_eq!(ev.content.currently_active, Some(false));
        assert_eq!(ev.content.displayname, None);
        assert_eq!(ev.content.last_active_ago, Some(uint!(2_478_593)));
        assert_eq!(ev.content.presence, PresenceState::Online);
        #[cfg(not(feature = "unstable-msc4532"))]
        assert_eq!(ev.content.status_msg.as_deref(), Some("Making cupcakes"));
        #[cfg(feature = "unstable-msc4532")]
        assert_eq!(ev.content.status.msg.as_deref(), Some("Making cupcakes"));
        assert_eq!(ev.sender, "@example:localhost");

        #[cfg(feature = "compat-empty-string-null")]
        {
            let json = json!({
                "content": {
                    "avatar_url": "",
                    "currently_active": false,
                    "last_active_ago": 2_478_593,
                    "presence": "online",
                    "status_msg": "Making cupcakes"
                },
                "sender": "@example:localhost",
                "type": "m.presence"
            });

            let ev = from_json_value::<PresenceEvent>(json).unwrap();
            assert_eq!(ev.content.avatar_url, None);
            assert_eq!(ev.content.currently_active, Some(false));
            assert_eq!(ev.content.displayname, None);
            assert_eq!(ev.content.last_active_ago, Some(uint!(2_478_593)));
            assert_eq!(ev.content.presence, PresenceState::Online);
            #[cfg(not(feature = "unstable-msc4532"))]
            assert_eq!(ev.content.status_msg.as_deref(), Some("Making cupcakes"));
            #[cfg(feature = "unstable-msc4532")]
            assert_eq!(ev.content.status.msg.as_deref(), Some("Making cupcakes"));
            assert_eq!(ev.sender, "@example:localhost");
        }
    }
}
