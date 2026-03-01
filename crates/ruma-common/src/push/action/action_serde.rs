use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue as RawJsonValue;

use super::{Action, CustomAction, Tweak};
use crate::serde::from_raw_json_value;

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
                #[cfg(feature = "unstable-msc3768")]
                "org.matrix.msc3768.notify_in_app" => Ok(Action::NotifyInApp),
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
            #[cfg(feature = "unstable-msc3768")]
            Action::NotifyInApp => {
                serializer.serialize_unit_variant("Action", 0, "org.matrix.msc3768.notify_in_app")
            }
            Action::SetTweak(kind) => kind.serialize(serializer),
            Action::_Custom(custom) => custom.serialize(serializer),
        }
    }
}

/// Values for the `set_tweak` action.
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub(super) enum TweakSerdeHelper {
    Sound(SoundTweakSerdeHelper),
    Highlight(HighlightTweakSerdeHelper),
    Custom {
        #[serde(rename = "set_tweak")]
        name: String,
        value: Box<RawJsonValue>,
    },
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "set_tweak", rename = "sound")]
pub(super) struct SoundTweakSerdeHelper {
    value: String,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "set_tweak", rename = "highlight")]
pub(super) struct HighlightTweakSerdeHelper {
    #[serde(default = "crate::serde::default_true", skip_serializing_if = "crate::serde::is_true")]
    value: bool,
}

impl From<Tweak> for TweakSerdeHelper {
    fn from(tweak: Tweak) -> Self {
        match tweak {
            Tweak::Sound(value) => Self::Sound(SoundTweakSerdeHelper { value }),
            Tweak::Highlight(value) => Self::Highlight(HighlightTweakSerdeHelper { value }),
            Tweak::Custom { name, value } => Self::Custom { name, value },
        }
    }
}

impl From<TweakSerdeHelper> for Tweak {
    fn from(tweak: TweakSerdeHelper) -> Self {
        match tweak {
            TweakSerdeHelper::Sound(SoundTweakSerdeHelper { value }) => Self::Sound(value),
            TweakSerdeHelper::Highlight(HighlightTweakSerdeHelper { value }) => {
                Self::Highlight(value)
            }
            TweakSerdeHelper::Custom { name, value } => Self::Custom { name, value },
        }
    }
}
