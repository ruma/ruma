use as_variant::as_variant;
use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize, de};
use serde_json::value::RawValue as RawJsonValue;

mod action_serde;

use crate::{
    PrivOwnedStr,
    serde::{JsonObject, from_raw_json_value},
};

/// This represents the different actions that should be taken when a rule is matched, and
/// controls how notifications are delivered to the client.
///
/// See [the spec](https://spec.matrix.org/latest/client-server-api/#actions) for details.
#[derive(Clone, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum Action {
    /// Causes matching events to generate a notification (both in-app and remote / push).
    Notify,

    /// Causes matching events to generate an in-app notification but no remote / push
    /// notification.
    #[cfg(feature = "unstable-msc3768")]
    NotifyInApp,

    /// Sets an entry in the 'tweaks' dictionary sent to the push gateway.
    SetTweak(Tweak),

    /// An unknown action.
    #[doc(hidden)]
    _Custom(CustomAction),
}

impl Action {
    /// Creates a new `Action`.
    ///
    /// Prefer to use the public variants of `Action` where possible; this constructor is meant
    /// be used for unsupported actions only and does not allow setting arbitrary data for
    /// supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the action type is known and deserialization of `data` to the
    /// corresponding variant fails.
    pub fn new(data: CustomActionData) -> serde_json::Result<Self> {
        Ok(match data {
            CustomActionData::String(s) => match s.as_str() {
                "notify" => Self::Notify,
                #[cfg(feature = "unstable-msc3768")]
                "org.matrix.msc3768.notify_in_app" => Self::NotifyInApp,
                _ => Self::_Custom(CustomAction(CustomActionData::String(s))),
            },
            CustomActionData::Object(o) => {
                if o.contains_key("set_tweak") {
                    Self::SetTweak(serde_json::from_value(o.into())?)
                } else {
                    Self::_Custom(CustomAction(CustomActionData::Object(o)))
                }
            }
        })
    }

    /// Whether this action is an `Action::SetTweak(Tweak::Highlight(true))`.
    pub fn is_highlight(&self) -> bool {
        matches!(self, Action::SetTweak(Tweak::Highlight(HighlightTweakValue::Yes)))
    }

    /// Whether this action should trigger a notification (either in-app or remote / push).
    pub fn should_notify(&self) -> bool {
        match self {
            Action::Notify => true,
            #[cfg(feature = "unstable-msc3768")]
            Action::NotifyInApp => true,
            _ => false,
        }
    }

    /// Whether this action should trigger a remote / push notification.
    #[cfg(feature = "unstable-msc3768")]
    pub fn should_notify_remote(&self) -> bool {
        matches!(self, Action::Notify)
    }

    /// The sound that should be played with this action, if any.
    pub fn sound(&self) -> Option<&SoundTweakValue> {
        as_variant!(self, Action::SetTweak(Tweak::Sound(sound)) => sound)
    }

    /// Access the data if this is a custom action.
    pub fn custom_data(&self) -> Option<&CustomActionData> {
        as_variant!(self, Self::_Custom).map(|action| &action.0)
    }
}

/// A custom action.
#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CustomAction(CustomActionData);

/// The data of a custom action.
#[allow(unknown_lints, unnameable_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomActionData {
    /// A string.
    String(String),

    /// An object.
    Object(JsonObject),
}

/// The `set_tweak` action.
#[derive(Clone, Debug)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum Tweak {
    /// The sound to be played when this notification arrives.
    ///
    /// A device may choose to alert the user by some other means if appropriate, eg. vibration.
    Sound(SoundTweakValue),

    /// A boolean representing whether or not this message should be highlighted in the UI.
    Highlight(HighlightTweakValue),

    #[doc(hidden)]
    _Custom(CustomTweak),
}

impl Tweak {
    /// Creates a new `Tweak`.
    ///
    /// Prefer to use the public variants of `Tweak` where possible; this constructor is meant
    /// be used for unsupported tweaks only and does not allow setting arbitrary data for
    /// supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `set_tweak` is known and deserialization of `value` to the
    /// corresponding variant fails.
    pub fn new(set_tweak: String, value: Option<Box<RawJsonValue>>) -> serde_json::Result<Self> {
        Ok(match set_tweak.as_str() {
            "sound" => Self::Sound(from_raw_json_value(
                &value.ok_or_else(|| de::Error::missing_field("value"))?,
            )?),
            "highlight" => {
                let value =
                    value.map(|value| from_raw_json_value::<bool, _>(&value)).transpose()?;

                let highlight = if value.is_none_or(|value| value) {
                    HighlightTweakValue::Yes
                } else {
                    HighlightTweakValue::No
                };

                Self::Highlight(highlight)
            }
            _ => Self::_Custom(CustomTweak { set_tweak, value }),
        })
    }

    /// Access the `set_tweak` value.
    pub fn set_tweak(&self) -> &str {
        match self {
            Self::Sound(_) => "sound",
            Self::Highlight(_) => "highlight",
            Self::_Custom(CustomTweak { set_tweak, .. }) => set_tweak,
        }
    }

    /// Access the value, if it is a custom tweak.
    pub fn custom_value(&self) -> Option<&RawJsonValue> {
        as_variant!(self, Self::_Custom).and_then(|tweak| tweak.value.as_deref())
    }
}

impl From<SoundTweakValue> for Tweak {
    fn from(value: SoundTweakValue) -> Self {
        Self::Sound(value)
    }
}

impl From<HighlightTweakValue> for Tweak {
    fn from(value: HighlightTweakValue) -> Self {
        Self::Highlight(value)
    }
}

/// A sound to play when a notification arrives.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "lowercase")]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum SoundTweakValue {
    /// Play the default notification sound.
    Default,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// Whether or not a message should be highlighted in the UI.
///
/// This will normally take the form of presenting the message in a different color and/or
/// style. The UI might also be adjusted to draw particular attention to the room in which the
/// event occurred.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::exhaustive_enums)]
pub enum HighlightTweakValue {
    /// Highlight the message.
    #[default]
    Yes,

    /// Don't highlight the message.
    No,
}

impl From<bool> for HighlightTweakValue {
    fn from(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }
}

/// A custom tweak.
#[doc(hidden)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomTweak {
    /// The kind of the custom tweak.
    set_tweak: String,

    /// The value of the custom tweak.
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Box<RawJsonValue>>,
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json};

    use super::{Action, HighlightTweakValue, SoundTweakValue, Tweak};
    use crate::assert_to_canonical_json_eq;

    #[test]
    fn serialize_notify() {
        assert_to_canonical_json_eq!(Action::Notify, json!("notify"));
    }

    #[cfg(feature = "unstable-msc3768")]
    #[test]
    fn serialize_notify_in_app() {
        assert_to_canonical_json_eq!(
            Action::NotifyInApp,
            json!("org.matrix.msc3768.notify_in_app"),
        );
    }

    #[test]
    fn serialize_tweak_sound() {
        assert_to_canonical_json_eq!(
            Action::SetTweak(Tweak::Sound(SoundTweakValue::Default)),
            json!({ "set_tweak": "sound", "value": "default" })
        );
    }

    #[test]
    fn serialize_tweak_highlight() {
        assert_to_canonical_json_eq!(
            Action::SetTweak(Tweak::Highlight(HighlightTweakValue::Yes)),
            json!({ "set_tweak": "highlight" })
        );

        assert_to_canonical_json_eq!(
            Action::SetTweak(Tweak::Highlight(HighlightTweakValue::No)),
            json!({ "set_tweak": "highlight", "value": false })
        );
    }

    #[test]
    fn deserialize_notify() {
        assert_matches!(from_json_value::<Action>(json!("notify")), Ok(Action::Notify));
    }

    #[cfg(feature = "unstable-msc3768")]
    #[test]
    fn deserialize_notify_in_app() {
        assert_matches!(
            from_json_value::<Action>(json!("org.matrix.msc3768.notify_in_app")),
            Ok(Action::NotifyInApp)
        );
    }

    #[test]
    fn deserialize_tweak_sound() {
        let json_data = json!({
            "set_tweak": "sound",
            "value": "default"
        });
        assert_matches!(
            from_json_value::<Action>(json_data),
            Ok(Action::SetTweak(Tweak::Sound(value)))
        );
        assert_eq!(value, SoundTweakValue::Default);

        let json_data = json!({
            "set_tweak": "sound",
            "value": "custom"
        });
        assert_matches!(
            from_json_value::<Action>(json_data),
            Ok(Action::SetTweak(Tweak::Sound(value)))
        );
        assert_eq!(value.as_str(), "custom");
    }

    #[test]
    fn deserialize_tweak_highlight() {
        let json_data = json!({
            "set_tweak": "highlight",
            "value": true
        });
        assert_matches!(
            from_json_value::<Action>(json_data),
            Ok(Action::SetTweak(Tweak::Highlight(HighlightTweakValue::Yes)))
        );
    }

    #[test]
    fn deserialize_tweak_highlight_with_default_value() {
        assert_matches!(
            from_json_value::<Action>(json!({ "set_tweak": "highlight" })),
            Ok(Action::SetTweak(Tweak::Highlight(HighlightTweakValue::Yes)))
        );
    }
}
