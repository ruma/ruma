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

    /// Whether the user is currently active or not.
    ///
    /// If the `unstable-msc4532` feature is enabled and `presence` is a [MSC4532]
    /// presence state, this field will be ignored and will always have the same value
    /// as [`PresenceState::currently_active`] after deserialization.
    #[cfg_attr(
        feature = "unstable-msc4532",
        deprecated(
            note = "Deprecated when MSC4532 is enabled, use `PresenceState::currently_active` instead"
        )
    )]
    pub currently_active: Option<bool>,

    /// The current display name for this user.
    pub displayname: Option<String>,

    /// The last time since this user performed some action, in milliseconds.
    pub last_active_ago: Option<UInt>,

    /// The presence state for this user.
    pub presence: PresenceState,

    /// An optional description to accompany the presence.
    ///
    /// If the `unstable-msc4532` feature is enabled, this field is ignored during
    /// serialization, and will always have the same value as `status.msg` after
    /// deserialization.
    #[cfg_attr(
        feature = "unstable-msc4532",
        deprecated(note = "Deprecated when MSC4532 is enabled, use `status` instead")
    )]
    pub status_msg: Option<String>,

    /// Optional information to accompany the presence.
    ///
    /// This field uses the unstable prefix defined in [MSC4532].
    ///
    /// If this field is not present at deserialization, the value of `status_msg`
    /// will be used instead.
    ///
    /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
    #[cfg(feature = "unstable-msc4532")]
    pub status: PresenceStatus,
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
    avatar_url: Option<OwnedMxcUri>,

    /// Whether the user is currently active or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    currently_active: Option<bool>,

    /// The current display name for this user.
    #[serde(skip_serializing_if = "Option::is_none")]
    displayname: Option<String>,

    /// The last time since this user performed some action, in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    last_active_ago: Option<UInt>,

    /// The presence state for this user.
    presence: PresenceState,

    /// An optional description to accompany the presence.
    #[serde(skip_serializing_if = "Option::is_none")]
    status_msg: Option<String>,

    /// Optional information to accompany the presence.
    ///
    /// This field uses the unstable prefix defined in [MSC4532].
    ///
    /// If this field is not present at deserialization, the value of `status_msg`
    /// will be used instead.
    ///
    /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
    #[cfg(feature = "unstable-msc4532")]
    #[serde(
        skip_serializing_if = "ruma_common::serde::is_default",
        rename = "org.continuwuity.presence_v2.msc4532.status",
        default
    )]
    status: PresenceStatus,
}

impl<'de> Deserialize<'de> for PresenceEventContent {
    #[allow(deprecated, unused_variables)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let PresenceEventRepr {
            avatar_url,
            currently_active,
            displayname,
            last_active_ago,
            presence,
            status_msg,
            #[cfg(feature = "unstable-msc4532")]
            status,
        } = PresenceEventRepr::deserialize(deserializer)?;

        #[cfg(feature = "unstable-msc4532")]
        let status_msg = if status == PresenceStatus::default() { status_msg } else { status.msg };
        #[cfg(feature = "unstable-msc4532")]
        let currently_active = if presence.is_msc4532_state() {
            // ignore the provided currently_active value and use the proposal's definition
            Some(presence.currently_active())
        } else {
            currently_active
        };

        Ok(Self {
            avatar_url,
            currently_active,
            displayname,
            last_active_ago,
            presence,
            #[cfg(feature = "unstable-msc4532")]
            status: PresenceStatus::new(status_msg.clone()),
            status_msg,
        })
    }
}

#[allow(deprecated, unused_variables)]
impl Serialize for PresenceEventContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let PresenceEventContent {
            avatar_url,
            currently_active,
            displayname,
            last_active_ago,
            presence,
            #[allow(unused_variables)]
            status_msg,
            #[cfg(feature = "unstable-msc4532")]
            status,
        } = self;

        PresenceEventRepr {
            #[cfg(not(feature = "unstable-msc4532"))]
            currently_active: *currently_active,
            #[cfg(feature = "unstable-msc4532")]
            currently_active: Some(presence.currently_active()),

            #[cfg(not(feature = "unstable-msc4532"))]
            status_msg: status_msg.clone(),
            #[cfg(feature = "unstable-msc4532")]
            status_msg: status.msg.clone(),
            #[cfg(feature = "unstable-msc4532")]
            status: status.clone(),

            avatar_url: avatar_url.clone(),
            last_active_ago: *last_active_ago,
            displayname: displayname.clone(),
            presence: presence.clone(),
        }
        .serialize(serializer)
    }
}

impl PresenceEventContent {
    /// Creates a new `PresenceEventContent` with the given state.
    pub fn new(presence: PresenceState) -> Self {
        #[allow(deprecated)]
        Self {
            avatar_url: None,
            currently_active: None,
            displayname: None,
            last_active_ago: None,
            presence,
            status_msg: None,
            #[cfg(feature = "unstable-msc4532")]
            status: PresenceStatus::default(),
        }
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use js_int::uint;
    #[cfg(feature = "unstable-msc4532")]
    use ruma_common::presence::PresenceStatus;
    use ruma_common::{
        canonical_json::assert_to_canonical_json_eq, owned_mxc_uri, presence::PresenceState,
    };
    use serde_json::{from_value as from_json_value, json};
    use strass::assert_variant_eq;

    use super::{PresenceEvent, PresenceEventContent};

    #[test]
    fn serialization() {
        let content = PresenceEventContent {
            avatar_url: Some(owned_mxc_uri!("mxc://localhost/wefuiwegh8742w")),
            currently_active: Some(true),
            displayname: None,
            last_active_ago: Some(uint!(2_478_593)),
            presence: PresenceState::Online,
            #[allow(deprecated)]
            status_msg: Some("Making cupcakes".into()),
            #[cfg(feature = "unstable-msc4532")]
            status: PresenceStatus::new(Some("Making cupcakes".into())),
        };

        #[cfg(feature = "unstable-msc4532")]
        assert_to_canonical_json_eq!(
            content,
            json!({
                "avatar_url": "mxc://localhost/wefuiwegh8742w",
                "currently_active": true,
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
                "currently_active": true,
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
                "currently_active": true,
                "last_active_ago": 2_478_593,
                "presence": "online",
                "status_msg": "Making cupcakes"
            },
            "sender": "@example:localhost",
            "type": "m.presence"
        });

        let ev = from_json_value::<PresenceEvent>(json).unwrap();
        assert_variant_eq!(ev.content.avatar_url, Some("mxc://localhost/wefuiwegh8742w"));
        assert_eq!(ev.content.currently_active, Some(true));
        assert_eq!(ev.content.displayname, None);
        assert_eq!(ev.content.last_active_ago, Some(uint!(2_478_593)));
        assert_eq!(ev.content.presence, PresenceState::Online);
        assert_eq!(ev.content.status_msg.as_deref(), Some("Making cupcakes"));
        #[cfg(feature = "unstable-msc4532")]
        assert_eq!(ev.content.status.msg.as_deref(), Some("Making cupcakes"));
        assert_eq!(ev.sender, "@example:localhost");

        #[cfg(feature = "compat-empty-string-null")]
        {
            let json = json!({
                "content": {
                    "avatar_url": "",
                    "currently_active": true,
                    "last_active_ago": 2_478_593,
                    "presence": "online",
                    "status_msg": "Making cupcakes"
                },
                "sender": "@example:localhost",
                "type": "m.presence"
            });

            let ev = from_json_value::<PresenceEvent>(json).unwrap();
            assert_eq!(ev.content.avatar_url, None);
            assert_eq!(ev.content.currently_active, Some(true));
            assert_eq!(ev.content.displayname, None);
            assert_eq!(ev.content.last_active_ago, Some(uint!(2_478_593)));
            assert_eq!(ev.content.presence, PresenceState::Online);
            assert_eq!(ev.content.status_msg.as_deref(), Some("Making cupcakes"));
            #[cfg(feature = "unstable-msc4532")]
            assert_eq!(ev.content.status.msg.as_deref(), Some("Making cupcakes"));
            assert_eq!(ev.sender, "@example:localhost");
        }
    }
}
