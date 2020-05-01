use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as RawJsonValue;

/// Values for the `set_tweak` action.
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Tweak {
    Sound(SoundTweak),
    Highlight(HighlightTweak),
    Custom {
        #[serde(rename = "set_tweak")]
        name: String,
        value: Box<RawJsonValue>,
    },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "set_tweak", rename = "sound")]
pub struct SoundTweak {
    value: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "set_tweak", rename = "highlight")]
pub struct HighlightTweak {
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
