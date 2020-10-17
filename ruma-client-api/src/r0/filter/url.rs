use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

/// Options for filtering based on the presence of a URL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UrlFilter {
    /// Includes only events with a url key in their content.
    EventsWithUrl,

    /// Excludes events with a url key in their content.
    EventsWithoutUrl,
}

impl Serialize for UrlFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Self::EventsWithUrl => serializer.serialize_bool(true),
            Self::EventsWithoutUrl => serializer.serialize_bool(false),
        }
    }
}

impl<'de> Deserialize<'de> for UrlFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match bool::deserialize(deserializer)? {
            true => Self::EventsWithUrl,
            false => Self::EventsWithoutUrl,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::UrlFilter;

    #[test]
    fn serialize_filter_events_with_url() {
        let events_with_url = UrlFilter::EventsWithUrl;
        assert_eq!(to_json_value(events_with_url).unwrap(), json!(true))
    }

    #[test]
    fn serialize_filter_events_without_url() {
        let events_without_url = UrlFilter::EventsWithoutUrl;
        assert_eq!(to_json_value(events_without_url).unwrap(), json!(false))
    }

    #[test]
    fn deserialize_filter_events_with_url() {
        let json = json!(true);
        assert_eq!(from_json_value::<UrlFilter>(json).unwrap(), UrlFilter::EventsWithUrl);
    }

    #[test]
    fn deserialize_filter_events_without_url() {
        let json = json!(false);
        assert_eq!(from_json_value::<UrlFilter>(json).unwrap(), UrlFilter::EventsWithoutUrl);
    }
}
