use serde::{Deserialize, Deserializer, Serialize, Serializer, de, ser::SerializeStruct};

use super::{Action, CustomActionData, CustomTweak, HighlightTweakValue, Tweak};

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(CustomActionData::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Action::Notify => serializer.serialize_str("notify"),
            #[cfg(feature = "unstable-msc3768")]
            Action::NotifyInApp => serializer.serialize_str("org.matrix.msc3768.notify_in_app"),
            Action::SetTweak(kind) => kind.serialize(serializer),
            Action::_Custom(custom) => custom.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Tweak {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let CustomTweak { set_tweak, value } = CustomTweak::deserialize(deserializer)?;
        Self::new(set_tweak, value).map_err(de::Error::custom)
    }
}

impl Serialize for Tweak {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Sound(tweak) => {
                let mut s = serializer.serialize_struct("Tweak", 2)?;
                s.serialize_field("set_tweak", &"sound")?;
                s.serialize_field("value", tweak)?;
                s.end()
            }
            Self::Highlight(tweak) => {
                let is_no_highlight = *tweak == HighlightTweakValue::No;
                let len = if is_no_highlight { 2 } else { 1 };

                let mut s = serializer.serialize_struct("Tweak", len)?;
                s.serialize_field("set_tweak", &"highlight")?;

                if is_no_highlight {
                    s.serialize_field("value", &false)?;
                }

                s.end()
            }
            Self::_Custom(tweak) => tweak.serialize(serializer),
        }
    }
}
