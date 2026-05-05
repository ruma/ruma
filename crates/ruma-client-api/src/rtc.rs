//! [MatrixRTC] endpoints.
//!
//! [MatrixRTC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4143

use std::borrow::Cow;

use ruma_common::serde::JsonObject;
#[cfg(feature = "unstable-msc4195")]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub mod transports;

/// Information about a specific MatrixRTC transport.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "type")]
pub enum RtcTransport {
    /// A LiveKit RTC transport.
    #[cfg(feature = "unstable-msc4195")]
    #[serde(rename = "livekit")]
    LiveKit(LiveKitRtcTransport),

    /// A custom RTC transport.
    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomRtcTransport),
}

impl RtcTransport {
    /// A constructor to create a custom RTC transport.
    ///
    /// Prefer to use the public variants of `RtcTransport` where possible; this constructor is
    /// meant to be used for unsupported transport types only and does not allow setting arbitrary
    /// data for supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `transport_type` is known and serialization of `data` to the
    /// corresponding `RtcTransport` variant fails.
    pub fn new(transport_type: &str, data: JsonObject) -> serde_json::Result<Self> {
        #[cfg(feature = "unstable-msc4195")]
        fn deserialize_variant<T: DeserializeOwned>(obj: JsonObject) -> serde_json::Result<T> {
            use serde_json::Value as JsonValue;

            serde_json::from_value(JsonValue::Object(obj))
        }

        Ok(match transport_type {
            #[cfg(feature = "unstable-msc4195")]
            "livekit" => Self::LiveKit(deserialize_variant(data)?),
            _ => Self::_Custom(CustomRtcTransport {
                transport_type: transport_type.to_owned(),
                data,
            }),
        })
    }

    #[cfg(feature = "unstable-msc4195")]
    /// Creates a new `RtcTransport::LiveKit`.
    pub fn livekit(service_url: String) -> Self {
        Self::LiveKit(LiveKitRtcTransport { service_url })
    }

    /// Returns a reference to the transport type of this RTC transport.
    pub fn transport_type(&self) -> &str {
        match self {
            #[cfg(feature = "unstable-msc4195")]
            Self::LiveKit(_) => "livekit",
            Self::_Custom(custom) => &custom.transport_type,
        }
    }

    /// Returns the associated data.
    ///
    /// The returned JSON object won't contain the `type` field, please use
    /// [`.transport_type()`][Self::transport_type] to access that.
    ///
    /// Prefer to use the public variants of `RtcTransport` where possible; this method is meant
    /// to be used for custom transport types only.
    pub fn data(&self) -> Cow<'_, JsonObject> {
        #[cfg(feature = "unstable-msc4195")]
        fn serialize<T: Serialize>(object: &T) -> JsonObject {
            use serde_json::Value as JsonValue;

            match serde_json::to_value(object).expect("rtc transport type serialization to succeed")
            {
                JsonValue::Object(object) => object,
                _ => panic!("all rtc transport types must serialize to objects"),
            }
        }

        match self {
            #[cfg(feature = "unstable-msc4195")]
            Self::LiveKit(info) => Cow::Owned(serialize(info)),
            Self::_Custom(info) => Cow::Borrowed(&info.data),
        }
    }
}

/// Information about a LiveKit RTC transport.
#[cfg(feature = "unstable-msc4195")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct LiveKitRtcTransport {
    /// The URL for the LiveKit service.
    #[serde(rename = "livekit_service_url")]
    pub service_url: String,
}

/// Information about a custom RTC transport.
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CustomRtcTransport {
    /// The type of RTC transport.
    #[serde(rename = "type")]
    transport_type: String,

    /// Remaining RTC transport data.
    #[serde(flatten)]
    data: JsonObject,
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use serde_json::{
        Value as JsonValue, from_value as from_json_value, json, to_value as to_json_value,
    };

    use super::RtcTransport;

    #[test]
    fn serialize_roundtrip_custom_rtc_transport() {
        let transport_type = "local.custom.transport";
        assert_matches!(
            json!({
                "foo": "bar",
                "baz": true,
            }),
            JsonValue::Object(transport_data)
        );
        let transport = RtcTransport::new(transport_type, transport_data.clone()).unwrap();
        let json = json!({
            "type": transport_type,
            "foo": "bar",
            "baz": true,
        });

        assert_eq!(transport.transport_type(), transport_type);
        assert_eq!(*transport.data().as_ref(), transport_data);
        assert_eq!(to_json_value(&transport).unwrap(), json);
        assert_eq!(from_json_value::<RtcTransport>(json).unwrap(), transport);
    }

    #[cfg(feature = "unstable-msc4195")]
    #[test]
    fn serialize_roundtrip_livekit_sfu_transport() {
        let transport_type = "livekit";
        let livekit_service_url = "http://livekit.local/";
        let transport = RtcTransport::livekit(livekit_service_url.to_owned());
        let json = json!({
            "type": transport_type,
            "livekit_service_url": livekit_service_url,
        });

        assert_eq!(transport.transport_type(), transport_type);
        assert_eq!(to_json_value(&transport).unwrap(), json);
        assert_eq!(from_json_value::<RtcTransport>(json).unwrap(), transport);
    }
}
