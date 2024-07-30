use std::collections::BTreeMap;

use as_variant::as_variant;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::{RawValue as RawJsonValue, Value as JsonValue};

use crate::serde::from_raw_json_value;

/// This represents the different actions that should be taken when a rule is matched, and
/// controls how notifications are delivered to the client.
///
/// See [the spec](https://spec.matrix.org/latest/client-server-api/#actions) for details.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum Action {
    /// Causes matching events to generate a notification.
    Notify,

    /// Sets an entry in the 'tweaks' dictionary sent to the push gateway.
    SetTweak(Tweak),

    /// An unknown action.
    #[doc(hidden)]
    _Custom(CustomAction),
}

impl Action {
    /// Whether this action is an `Action::SetTweak(Tweak::Highlight(true))`.
    pub fn is_highlight(&self) -> bool {
        matches!(self, Action::SetTweak(Tweak::Highlight(true)))
    }

    /// Whether this action should trigger a notification.
    pub fn should_notify(&self) -> bool {
        matches!(self, Action::Notify)
    }

    /// The sound that should be played with this action, if any.
    pub fn sound(&self) -> Option<&str> {
        as_variant!(self, Action::SetTweak(Tweak::Sound(sound)) => sound)
    }
}

/// The `set_tweak` action.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(from = "tweak_serde::Tweak", into = "tweak_serde::Tweak")]
pub enum Tweak {
    /// A string representing the sound to be played when this notification arrives.
    ///
    /// A value of "default" means to play a default sound. A device may choose to alert the user
    /// by some other means if appropriate, eg. vibration.
    Sound(String),

    /// A boolean representing whether or not this message should be highlighted in the UI.
    ///
    /// This will normally take the form of presenting the message in a different color and/or
    /// style. The UI might also be adjusted to draw particular attention to the room in which the
    /// event occurred. If a `highlight` tweak is given with no value, its value is defined to be
    /// `true`. If no highlight tweak is given at all then the value of `highlight` is defined to
    /// be `false`.
    Highlight(#[serde(default = "crate::serde::default_true")] bool),

    /// A custom tweak
    Custom {
        /// The name of the custom tweak (`set_tweak` field)
        name: String,

        /// The value of the custom tweak
        value: Box<RawJsonValue>,
    },
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let custom: CustomAction = from_raw_json_value(&json)?;

        match &custom {
            CustomAction::String(s) => match s.as_str() {
                "notify" => Ok(Action::Notify),
                _ => Ok(Action::_Custom(custom)),
            },
            CustomAction::Object(o) => {
                if o.get("set_tweak").is_some() {
                    Ok(Action::SetTweak(from_raw_json_value(&json)?))
                } else {
                    Ok(Action::_Custom(custom))
                }
            }
        }
    }
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Action::Notify => serializer.serialize_unit_variant("Action", 0, "notify"),
            Action::SetTweak(kind) => kind.serialize(serializer),
            Action::_Custom(custom) => custom.serialize(serializer),
        }
    }
}

/// An unknown action.
#[doc(hidden)]
#[allow(unknown_lints, unnameable_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CustomAction {
    /// A string.
    String(String),

    /// An object.
    Object(BTreeMap<String, JsonValue>),
}

mod tweak_serde {
    use serde::{Deserialize, Serialize};
    use serde_json::value::RawValue as RawJsonValue;

    /// Values for the `set_tweak` action.
    #[derive(Clone, Deserialize, Serialize)]
    #[serde(untagged)]
    pub(crate) enum Tweak {
        Sound(SoundTweak),
        Highlight(HighlightTweak),
        Custom {
            #[serde(rename = "set_tweak")]
            name: String,
            value: Box<RawJsonValue>,
        },
    }

    #[derive(Clone, PartialEq, Deserialize, Serialize)]
    #[serde(tag = "set_tweak", rename = "sound")]
    pub(crate) struct SoundTweak {
        value: String,
    }

    #[derive(Clone, PartialEq, Deserialize, Serialize)]
    #[serde(tag = "set_tweak", rename = "highlight")]
    pub(crate) struct HighlightTweak {
        #[serde(
            default = "crate::serde::default_true",
            skip_serializing_if = "crate::serde::is_true"
        )]
        value: bool,
    }

    impl From<super::Tweak> for Tweak {
        fn from(tweak: super::Tweak) -> Self {
            use super::Tweak::*;

            match tweak {
                Sound(value) => Self::Sound(SoundTweak { value }),
                Highlight(value) => Self::Highlight(HighlightTweak { value }),
                Custom { name, value } => Self::Custom { name, value },
            }
        }
    }

    impl From<Tweak> for super::Tweak {
        fn from(tweak: Tweak) -> Self {
            use Tweak::*;

            match tweak {
                Sound(SoundTweak { value }) => Self::Sound(value),
                Highlight(HighlightTweak { value }) => Self::Highlight(value),
                Custom { name, value } => Self::Custom { name, value },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{Action, Tweak};

    #[test]
    fn serialize_string() {
        assert_eq!(to_json_value(Action::Notify).unwrap(), json!("notify"));
    }

    #[test]
    fn serialize_tweak_sound() {
        assert_eq!(
            to_json_value(Action::SetTweak(Tweak::Sound("default".into()))).unwrap(),
            json!({ "set_tweak": "sound", "value": "default" })
        );
    }

    #[test]
    fn serialize_tweak_highlight() {
        assert_eq!(
            to_json_value(Action::SetTweak(Tweak::Highlight(true))).unwrap(),
            json!({ "set_tweak": "highlight" })
        );

        assert_eq!(
            to_json_value(Action::SetTweak(Tweak::Highlight(false))).unwrap(),
            json!({ "set_tweak": "highlight", "value": false })
        );
    }

    #[test]
    fn deserialize_string() {
        assert_matches!(from_json_value::<Action>(json!("notify")), Ok(Action::Notify));
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
        assert_eq!(value, "default");
    }

    #[test]
    fn deserialize_tweak_highlight() {
        let json_data = json!({
            "set_tweak": "highlight",
            "value": true
        });
        assert_matches!(
            from_json_value::<Action>(json_data),
            Ok(Action::SetTweak(Tweak::Highlight(true)))
        );
    }

    #[test]
    fn deserialize_tweak_highlight_with_default_value() {
        assert_matches!(
            from_json_value::<Action>(json!({ "set_tweak": "highlight" })),
            Ok(Action::SetTweak(Tweak::Highlight(true)))
        );
    }
}
