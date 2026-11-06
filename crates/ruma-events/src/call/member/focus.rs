//! Types for MatrixRTC Focus/SFU configurations.

use std::borrow::Cow;

use ruma_common::{OwnedServerName, serde::JsonObject};
use ruma_macros::StringEnum;
use serde::{Deserialize, Deserializer, Serialize};

use crate::PrivOwnedStr;

/// Description of the SFU/Focus a membership can be connected to.
///
/// A focus can be any server powering the MatrixRTC session (SFU,
/// MCU). It serves as a node to redistribute RTC streams.
#[allow(deprecated)]
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "type")]
pub enum Focus {
    /// A LiveKit focus, as defined in the merged state of [MSC4195].
    ///
    /// [MSC4195]: https://github.com/matrix-org/matrix-spec-proposals/pull/4195
    #[serde(rename = "m.livekit")]
    Livekit(LivekitFocus),

    /// A LiveKit focus with the `livekit` type used by implementations of earlier versions of
    /// [MSC4195].
    ///
    /// [MSC4195]: https://github.com/matrix-org/matrix-spec-proposals/pull/4195
    #[serde(rename = "livekit")]
    #[deprecated = "Use `Focus::Livekit` instead unless you require backwards compatibility with older versions of MSC4195."]
    LegacyLivekit(LivekitFocus),

    /// A custom focus.
    #[doc(hidden)]
    #[serde(untagged)]
    _Custom(CustomFocus),
}

impl<'de> Deserialize<'de> for Focus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let mut obj = JsonObject::deserialize(deserializer)?;
        let focus_type = match obj.remove("type") {
            Some(serde_json::Value::String(s)) => s,
            Some(_) => return Err(D::Error::custom("`type` must be a string")),
            None => return Err(D::Error::missing_field("type")),
        };

        Self::new(&focus_type, obj).map_err(D::Error::custom)
    }
}

impl Focus {
    /// A constructor to create a custom focus.
    ///
    /// Prefer to use the public variants of `Focus` where possible; this constructor is meant to
    /// be used for unsupported focus types only and does not allow setting arbitrary data for
    /// supported ones.
    ///
    /// # Errors
    ///
    /// Returns an error if the `focus_type` is known and deserialization of `data` to the
    /// corresponding `Focus` variant fails.
    pub fn new(focus_type: &str, data: JsonObject) -> serde_json::Result<Self> {
        Ok(match focus_type {
            "m.livekit" => Self::Livekit(serde_json::from_value(serde_json::Value::Object(data))?),
            #[allow(deprecated)]
            "livekit" => {
                Self::LegacyLivekit(serde_json::from_value(serde_json::Value::Object(data))?)
            }
            _ => Self::_Custom(CustomFocus { focus_type: focus_type.to_owned(), data }),
        })
    }

    /// Creates a new `Focus::Livekit` with the given SFU URL.
    pub fn livekit(url: String) -> Self {
        Self::Livekit(LivekitFocus::new(url))
    }

    /// Returns the type of this focus.
    pub fn focus_type(&self) -> &str {
        match self {
            Self::Livekit(_) => "m.livekit",
            #[allow(deprecated)]
            Self::LegacyLivekit(_) => "livekit",
            Self::_Custom(custom) => &custom.focus_type,
        }
    }

    /// Returns the associated data.
    ///
    /// The returned JSON object won't contain the `type` field, please use
    /// [`.focus_type()`][Self::focus_type] to access that.
    ///
    /// Prefer to use the public variants of `Focus` where possible; this method is meant to be
    /// used for custom focus types only.
    pub fn data(&self) -> Cow<'_, JsonObject> {
        match self {
            Self::Livekit(info) => Cow::Owned(serialize_focus_data(info)),
            #[allow(deprecated)]
            Self::LegacyLivekit(info) => Cow::Owned(serialize_focus_data(info)),
            Self::_Custom(info) => Cow::Borrowed(&info.data),
        }
    }
}

/// Serializes the data of a known focus type to a JSON object.
fn serialize_focus_data<T: Serialize>(object: &T) -> JsonObject {
    use serde_json::Value as JsonValue;

    match serde_json::to_value(object).expect("focus type serialization to succeed") {
        JsonValue::Object(object) => object,
        _ => panic!("all focus types must serialize to objects"),
    }
}

/// The struct to describe LiveKit as a `preferred_foci`.
///
/// This is used for both the `m.livekit` and the legacy `livekit` focus types. Implementations of
/// earlier versions of [MSC4195] may omit the `url` and use `livekit_service_url` instead,
/// independently of the focus type.
///
/// [MSC4195]: https://github.com/matrix-org/matrix-spec-proposals/pull/4195
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct LivekitFocus {
    /// The alias where the LiveKit sessions can be reached.
    #[serde(rename = "livekit_alias", skip_serializing_if = "Option::is_none")]
    #[deprecated = "Newer versions of Element Call won't fill this field anymore."]
    pub alias: Option<String>,

    /// The WebSocket URL of the LiveKit SFU.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// The server name of the homeserver that grants access to the LiveKit SFU.
    ///
    /// This must be set when publishing on an SFU of a different homeserver than the sender's. If
    /// this is `None`, the SFU belongs to the sender's homeserver.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<OwnedServerName>,

    /// The URL of the JWT service for the LiveKit instance.
    #[serde(rename = "livekit_service_url", skip_serializing_if = "Option::is_none")]
    #[deprecated = "Use `url` instead unless you require backwards compatibility with older versions of MSC4195."]
    pub service_url: Option<String>,
}

impl LivekitFocus {
    /// Initialize a [`LivekitFocus`] with the given SFU URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL of the LiveKit SFU.
    #[allow(deprecated)]
    pub fn new(url: String) -> Self {
        Self { alias: None, url: Some(url), server_name: None, service_url: None }
    }

    /// Initialize a [`LivekitFocus`] with the given JWT service URL.
    ///
    /// # Arguments
    ///
    /// * `service_url` - The URL of the JWT service for the LiveKit instance.
    #[deprecated = "Use `LivekitFocus::new()` instead unless you require backwards compatibility with older versions of MSC4195."]
    #[allow(deprecated)]
    pub fn with_service_url(service_url: String) -> Self {
        Self { alias: None, url: None, server_name: None, service_url: Some(service_url) }
    }
}

/// Information about a custom focus.
///
/// This type does not implement `Deserialize` to prevent users from constructing the `_Custom`
/// variant of [`Focus`] for a known `type`. Deserialize through [`Focus`] instead.
#[doc(hidden)]
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CustomFocus {
    /// The type of focus.
    #[serde(rename = "type")]
    focus_type: String,

    /// Remaining focus data.
    #[serde(flatten)]
    data: JsonObject,
}

/// Data to define the actively used Focus.
///
/// A focus can be any server powering the MatrixRTC session (SFU,
/// MCU). It serves as a node to redistribute RTC streams.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActiveFocus {
    /// LiveKit is one possible type of SFU/Focus that can be used for a MatrixRTC session.
    Livekit(ActiveLivekitFocus),
}

/// The fields to describe the `active_foci`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct ActiveLivekitFocus {
    /// The selection method used to select the LiveKit focus for the rtc session.
    pub focus_selection: FocusSelection,
}

impl ActiveLivekitFocus {
    /// Initialize a [`ActiveLivekitFocus`].
    ///
    /// # Arguments
    ///
    /// * `focus_selection` - The selection method used to select the LiveKit focus for the rtc
    ///   session.
    pub fn new() -> Self {
        Self { focus_selection: FocusSelection::OldestMembership }
    }
}

/// How to select the active focus for LiveKit
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum FocusSelection {
    /// Select the active focus by using the oldest membership and the oldest focus.
    OldestMembership,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
