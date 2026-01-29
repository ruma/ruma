//! `GET /.well-known/matrix/client` ([spec])
//!
//! [spec]: https://spec.matrix.org/latest/client-server-api/#getwell-knownmatrixclient
//!
//! Get discovery information about the domain.

#[cfg(feature = "unstable-msc4143")]
use std::borrow::Cow;

#[cfg(feature = "unstable-msc4143")]
use ruma_common::serde::JsonObject;
use ruma_common::{
    api::{auth_scheme::NoAuthentication, request, response},
    metadata,
};
#[cfg(feature = "unstable-msc4143")]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[cfg(feature = "unstable-msc4143")]
use serde_json::Value as JsonValue;

metadata! {
    method: GET,
    rate_limited: false,
    authentication: NoAuthentication,
    path: "/.well-known/matrix/client",
}

/// Request type for the `client_well_known` endpoint.
#[request(error = crate::Error)]
#[derive(Default)]
pub struct Request {}

/// Response type for the `client_well_known` endpoint.
#[response(error = crate::Error)]
pub struct Response {
    /// Information about the homeserver to connect to.
    #[serde(rename = "m.homeserver")]
    pub homeserver: HomeserverInfo,

    /// Information about the identity server to connect to.
    #[serde(rename = "m.identity_server", skip_serializing_if = "Option::is_none")]
    pub identity_server: Option<IdentityServerInfo>,

    /// Information about the tile server to use to display location data.
    #[cfg(feature = "unstable-msc3488")]
    #[serde(
        rename = "org.matrix.msc3488.tile_server",
        alias = "m.tile_server",
        skip_serializing_if = "Option::is_none"
    )]
    pub tile_server: Option<TileServerInfo>,

    /// A list of the available MatrixRTC foci, ordered by priority.
    #[cfg(feature = "unstable-msc4143")]
    #[serde(
        rename = "org.matrix.msc4143.rtc_foci",
        alias = "m.rtc_foci",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub rtc_foci: Vec<RtcFocusInfo>,
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Response {
    /// Creates a new `Response` with the given `HomeserverInfo`.
    pub fn new(homeserver: HomeserverInfo) -> Self {
        Self {
            homeserver,
            identity_server: None,
            #[cfg(feature = "unstable-msc3488")]
            tile_server: None,
            #[cfg(feature = "unstable-msc4143")]
            rtc_foci: Default::default(),
        }
    }
}

/// Information about a discovered homeserver.
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct HomeserverInfo {
    /// The base URL for the homeserver for client-server connections.
    pub base_url: String,
}

impl HomeserverInfo {
    /// Creates a new `HomeserverInfo` with the given `base_url`.
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

/// Information about a discovered identity server.
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct IdentityServerInfo {
    /// The base URL for the identity server for client-server connections.
    pub base_url: String,
}

impl IdentityServerInfo {
    /// Creates an `IdentityServerInfo` with the given `base_url`.
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

/// Information about a discovered map tile server.
#[cfg(feature = "unstable-msc3488")]
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct TileServerInfo {
    /// The URL of a map tile server's `style.json` file.
    ///
    /// See the [Mapbox Style Specification](https://docs.mapbox.com/mapbox-gl-js/style-spec/) for more details.
    pub map_style_url: String,
}

#[cfg(feature = "unstable-msc3488")]
impl TileServerInfo {
    /// Creates a `TileServerInfo` with the given map style URL.
    pub fn new(map_style_url: String) -> Self {
        Self { map_style_url }
    }
}

/// Information about a specific MatrixRTC focus.
#[cfg(feature = "unstable-msc4143")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "type")]
pub enum RtcFocusInfo {
    /// A LiveKit RTC focus.
    #[serde(rename = "livekit")]
    LiveKit(LiveKitRtcFocusInfo),

    /// A custom RTC focus.
    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomRtcFocusInfo),
}

#[cfg(feature = "unstable-msc4143")]
impl RtcFocusInfo {
    /// A constructor to create a custom RTC focus.
    ///
    /// Prefer to use the public variants of `RtcFocusInfo` where possible; this constructor is
    /// meant to be used for unsupported focus types only and does not allow setting arbitrary data
    /// for supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `focus_type` is known and serialization of `data` to the
    /// corresponding `RtcFocusInfo` variant fails.
    pub fn new(focus_type: &str, data: JsonObject) -> serde_json::Result<Self> {
        fn deserialize_variant<T: DeserializeOwned>(obj: JsonObject) -> serde_json::Result<T> {
            serde_json::from_value(JsonValue::Object(obj))
        }

        Ok(match focus_type {
            "livekit" => Self::LiveKit(deserialize_variant(data)?),
            _ => Self::_Custom(CustomRtcFocusInfo { focus_type: focus_type.to_owned(), data }),
        })
    }

    /// Creates a new `RtcFocusInfo::LiveKit`.
    pub fn livekit(service_url: String) -> Self {
        Self::LiveKit(LiveKitRtcFocusInfo { service_url })
    }

    /// Returns a reference to the focus type of this RTC focus.
    pub fn focus_type(&self) -> &str {
        match self {
            Self::LiveKit(_) => "livekit",
            Self::_Custom(custom) => &custom.focus_type,
        }
    }

    /// Returns the associated data.
    ///
    /// The returned JSON object won't contain the `focus_type` field, please use
    /// [`.focus_type()`][Self::focus_type] to access that.
    ///
    /// Prefer to use the public variants of `RtcFocusInfo` where possible; this method is meant to
    /// be used for custom focus types only.
    pub fn data(&self) -> Cow<'_, JsonObject> {
        fn serialize<T: Serialize>(object: &T) -> JsonObject {
            match serde_json::to_value(object).expect("rtc focus type serialization to succeed") {
                JsonValue::Object(object) => object,
                _ => panic!("all rtc focus types must serialize to objects"),
            }
        }

        match self {
            Self::LiveKit(info) => Cow::Owned(serialize(info)),
            Self::_Custom(info) => Cow::Borrowed(&info.data),
        }
    }
}

/// Information about a LiveKit RTC focus.
#[cfg(feature = "unstable-msc4143")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct LiveKitRtcFocusInfo {
    /// The URL for the LiveKit service.
    #[serde(rename = "livekit_service_url")]
    pub service_url: String,
}

/// Information about a custom RTC focus type.
#[doc(hidden)]
#[cfg(feature = "unstable-msc4143")]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CustomRtcFocusInfo {
    /// The type of RTC focus.
    #[serde(rename = "type")]
    focus_type: String,

    /// Remaining RTC focus data.
    #[serde(flatten)]
    data: JsonObject,
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "unstable-msc4143")]
    use assert_matches2::assert_matches;
    #[cfg(feature = "unstable-msc4143")]
    use ruma_common::canonical_json::assert_to_canonical_json_eq;
    #[cfg(feature = "unstable-msc4143")]
    use serde_json::{from_value as from_json_value, json};

    #[cfg(feature = "unstable-msc4143")]
    use super::RtcFocusInfo;

    #[test]
    #[cfg(feature = "unstable-msc4143")]
    fn test_livekit_rtc_focus_deserialization() {
        // Given the JSON for a LiveKit RTC focus.
        let json = json!({
            "type": "livekit",
            "livekit_service_url": "https://livekit.example.com"
        });

        // When deserializing it into an RtcFocusInfo.
        let focus: RtcFocusInfo = from_json_value(json).unwrap();

        // Then it should be recognized as a LiveKit focus with the correct service URL.
        assert_matches!(focus, RtcFocusInfo::LiveKit(info));
        assert_eq!(info.service_url, "https://livekit.example.com");
    }

    #[test]
    #[cfg(feature = "unstable-msc4143")]
    fn test_livekit_rtc_focus_serialization() {
        // Given a LiveKit RTC focus info.
        let focus = RtcFocusInfo::livekit("https://livekit.example.com".to_owned());

        // When serializing to JSON, it should match the expected JSON structure.
        assert_to_canonical_json_eq!(
            focus,
            json!({
                "type": "livekit",
                "livekit_service_url": "https://livekit.example.com"
            })
        );
    }

    #[test]
    #[cfg(feature = "unstable-msc4143")]
    fn test_custom_rtc_focus_serialization() {
        // Given the JSON for a custom RTC focus type with additional fields.
        let json = json!({
            "type": "some-focus-type",
            "additional-type-specific-field": "https://my_focus.domain",
            "another-additional-type-specific-field": ["with", "Array", "type"]
        });

        // When deserializing it into an RtcFocusInfo.
        let focus: RtcFocusInfo = from_json_value(json.clone()).unwrap();

        // Then it should be recognized as a custom focus type, with all the additional fields
        // included.
        assert_eq!(focus.focus_type(), "some-focus-type");

        let data = &focus.data();
        assert_eq!(data["additional-type-specific-field"], "https://my_focus.domain");

        let array_values: Vec<&str> = data["another-additional-type-specific-field"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(array_values, vec!["with", "Array", "type"]);

        assert!(!data.contains_key("type"));

        // When serializing it back to JSON, it should match the original JSON.
        assert_to_canonical_json_eq!(focus, json);
    }
}
