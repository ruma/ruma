use std::fmt::{self, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue as RawJsonValue;

/// This represents the different actions that should be taken when a rule is matched, and
/// controls how notifications are delivered to the client.
///
/// See <https://matrix.org/docs/spec/client_server/r0.6.0#actions> for details.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum Action {
    /// Causes matching events to generate a notification.
    Notify,

    /// Prevents matching events from generating a notification.
    DontNotify,

    /// Behaves like notify but homeservers may choose to coalesce multiple events
    /// into a single notification.
    Coalesce,

    /// Sets an entry in the 'tweaks' dictionary sent to the push gateway.
    SetTweak(Tweak),
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
    Highlight(#[serde(default = "ruma_serde::default_true")] bool),

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
        use serde::de::{MapAccess, Visitor};

        struct ActionVisitor;
        impl<'de> Visitor<'de> for ActionVisitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                write!(formatter, "a valid action object")
            }

            /// Match a simple action type
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "notify" => Ok(Action::Notify),
                    "dont_notify" => Ok(Action::DontNotify),
                    "coalesce" => Ok(Action::Coalesce),
                    s => Err(E::unknown_variant(&s, &["notify", "dont_notify", "coalesce"])),
                }
            }

            /// Match the more complex set_tweaks action object as a key-value map
            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                Tweak::deserialize(serde::de::value::MapAccessDeserializer::new(map))
                    .map(Action::SetTweak)
            }
        }

        deserializer.deserialize_any(ActionVisitor)
    }
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Action::Notify => serializer.serialize_unit_variant("Action", 0, "notify"),
            Action::DontNotify => serializer.serialize_unit_variant("Action", 1, "dont_notify"),
            Action::Coalesce => serializer.serialize_unit_variant("Action", 2, "coalesce"),
            Action::SetTweak(kind) => kind.serialize(serializer),
        }
    }
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
        #[serde(default = "ruma_serde::default_true", skip_serializing_if = "ruma_serde::is_true")]
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
    use super::{Action, Tweak};

    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    #[test]
    fn serialize_string() {
        assert_eq!(to_json_value(&Action::Notify).unwrap(), json!("notify"));
    }

    #[test]
    fn serialize_tweak_sound() {
        assert_eq!(
            to_json_value(&Action::SetTweak(Tweak::Sound("default".into()))).unwrap(),
            json!({ "set_tweak": "sound", "value": "default" })
        );
    }

    #[test]
    fn serialize_tweak_highlight() {
        assert_eq!(
            to_json_value(&Action::SetTweak(Tweak::Highlight(true))).unwrap(),
            json!({ "set_tweak": "highlight" })
        );

        assert_eq!(
            to_json_value(&Action::SetTweak(Tweak::Highlight(false))).unwrap(),
            json!({ "set_tweak": "highlight", "value": false })
        );
    }

    #[test]
    fn deserialize_string() {
        assert_matches!(from_json_value::<Action>(json!("notify")).unwrap(), Action::Notify);
    }

    #[test]
    fn deserialize_tweak_sound() {
        let json_data = json!({
            "set_tweak": "sound",
            "value": "default"
        });
        assert_matches!(
            &from_json_value::<Action>(json_data).unwrap(),
            Action::SetTweak(Tweak::Sound(value)) if value == "default"
        );
    }

    #[test]
    fn deserialize_tweak_highlight() {
        let json_data = json!({
            "set_tweak": "highlight",
            "value": true
        });
        assert_matches!(
            from_json_value::<Action>(json_data).unwrap(),
            Action::SetTweak(Tweak::Highlight(true))
        );
    }

    #[test]
    fn deserialize_tweak_highlight_with_default_value() {
        assert_matches!(
            from_json_value::<Action>(json!({ "set_tweak": "highlight" })).unwrap(),
            Action::SetTweak(Tweak::Highlight(true))
        );
    }
}
