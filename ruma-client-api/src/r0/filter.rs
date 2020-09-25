//! Endpoints for event filters.

pub mod create_filter;
pub mod get_filter;

use std::fmt;

use js_int::UInt;
use ruma_common::Outgoing;
use ruma_identifiers::{RoomId, UserId};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

/// Format to use for returned events
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventFormat {
    /// Client format, as described in the Client API.
    Client,

    /// Raw events from federation.
    Federation,
}

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

/// Filters to be applied to room events
#[derive(Clone, Copy, Debug, Default, Outgoing, Serialize)]
#[incoming_derive(Clone, Serialize)]
pub struct RoomEventFilter<'a> {
    /// A list of event types to exclude.
    ///
    /// If this list is absent then no event types are excluded. A matching type will be excluded
    /// even if it is listed in the 'types' filter. A '*' can be used as a wildcard to match any
    /// sequence of characters.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_types: &'a [String],

    /// A list of room IDs to exclude.
    ///
    /// If this list is absent then no rooms are excluded. A matching room will be excluded even if
    /// it is listed in the 'rooms' filter.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_rooms: &'a [String],

    /// The maximum number of events to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<UInt>,

    /// A list of room IDs to include.
    ///
    /// If this list is absent then all rooms are included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rooms: Option<&'a [RoomId]>,

    /// A list of sender IDs to exclude.
    ///
    /// If this list is absent then no senders are excluded. A matching sender will be excluded even
    /// if it is listed in the 'senders' filter.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_senders: &'a [UserId],

    /// A list of senders IDs to include.
    ///
    /// If this list is absent then all senders are included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub senders: Option<&'a [UserId]>,

    /// A list of event types to include.
    ///
    /// If this list is absent then all event types are included. A '*' can be used as a wildcard to
    /// match any sequence of characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<&'a [String]>,

    /// Controls whether to include events with a URL key in their content.
    ///
    /// * `None`: No filtering
    /// * `Some(EventsWithUrl)`: Only events with a URL
    /// * `Some(EventsWithoutUrl)`: Only events without a URL
    #[serde(rename = "contains_url", skip_serializing_if = "Option::is_none")]
    pub url_filter: Option<UrlFilter>,

    /// Options to control lazy-loading of membership events.
    #[serde(flatten)]
    pub lazy_load_options: LazyLoadOptions,
}

impl<'a> RoomEventFilter<'a> {
    /// A filter that can be used to ignore all room events
    pub fn ignore_all() -> Self {
        Self { types: Some(&[]), ..Default::default() }
    }
}

/// Filters to be applied to room data
#[derive(Clone, Copy, Debug, Default, Outgoing, Serialize)]
#[incoming_derive(Clone, Serialize)]
pub struct RoomFilter<'a> {
    /// Include rooms that the user has left in the sync.
    ///
    /// Defaults to `false`.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub include_leave: bool,

    /// The per user account data to include for rooms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_data: Option<RoomEventFilter<'a>>,

    /// The message and state update events to include for rooms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline: Option<RoomEventFilter<'a>>,

    /// The events that aren't recorded in the room history, e.g. typing and receipts, to include
    /// for rooms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<RoomEventFilter<'a>>,

    /// The state events to include for rooms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<RoomEventFilter<'a>>,

    /// A list of room IDs to exclude.
    ///
    /// If this list is absent then no rooms are excluded. A matching room will be excluded even if
    /// it is listed in the 'rooms' filter. This filter is applied before the filters in
    /// `ephemeral`, `state`, `timeline` or `account_data`.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_rooms: &'a [RoomId],

    /// A list of room IDs to include.
    ///
    /// If this list is absent then all rooms are included. This filter is applied before the
    /// filters in `ephemeral`, `state`, `timeline` or `account_data`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rooms: Option<&'a [RoomId]>,
}

impl<'a> RoomFilter<'a> {
    /// A filter that can be used to ignore all room events (of any type)
    pub fn ignore_all() -> Self {
        Self { rooms: Some(&[]), ..Default::default() }
    }
}

/// Filter for not-room data
#[derive(Clone, Copy, Debug, Default, Outgoing, Serialize)]
#[incoming_derive(Clone, Serialize)]
pub struct Filter<'a> {
    /// A list of event types to exclude.
    ///
    /// If this list is absent then no event types are excluded. A matching type will be excluded
    /// even if it is listed in the 'types' filter. A '*' can be used as a wildcard to match any
    /// sequence of characters.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_types: &'a [String],

    /// The maximum number of events to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<UInt>,

    /// A list of senders IDs to include.
    ///
    /// If this list is absent then all senders are included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub senders: Option<&'a [UserId]>,

    /// A list of event types to include.
    ///
    /// If this list is absent then all event types are included. A '*' can be used as a wildcard to
    /// match any sequence of characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<&'a [String]>,

    /// A list of sender IDs to exclude.
    ///
    /// If this list is absent then no senders are excluded. A matching sender will be excluded even
    /// if it is listed in the 'senders' filter.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_senders: &'a [UserId],
}

impl<'a> Filter<'a> {
    /// A filter that can be used to ignore all events
    pub fn ignore_all() -> Self {
        Self { types: Some(&[]), ..Default::default() }
    }
}

/// A filter definition
#[derive(Clone, Copy, Debug, Default, Outgoing, Serialize)]
#[incoming_derive(Clone, Serialize)]
pub struct FilterDefinition<'a> {
    /// List of event fields to include.
    ///
    /// If this list is absent then all fields are included. The entries may include '.' characters
    /// to indicate sub-fields. So ['content.body'] will include the 'body' field of the 'content'
    /// object. A literal '.' character in a field name may be escaped using a '\'. A server may
    /// include more fields than were requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_fields: Option<&'a [String]>,

    /// The format to use for events.
    ///
    /// 'client' will return the events in a format suitable for clients. 'federation' will return
    /// the raw event as received over federation. The default is 'client'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_format: Option<EventFormat>,

    /// The presence updates to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Filter<'a>>,

    /// The user account data that isn't associated with rooms to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_data: Option<Filter<'a>>,

    /// Filters to be applied to room data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room: Option<RoomFilter<'a>>,
}

impl<'a> FilterDefinition<'a> {
    /// A filter that can be used to ignore all events
    pub fn ignore_all() -> Self {
        Self {
            account_data: Some(Filter::ignore_all()),
            room: Some(RoomFilter::ignore_all()),
            presence: Some(Filter::ignore_all()),
            ..Default::default()
        }
    }
}

/// Specifies options for [lazy-loading membership events][lazy-loading] on
/// supported endpoints
///
/// [lazy-loading]: https://matrix.org/docs/spec/client_server/r0.6.0#lazy-loading-room-members
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LazyLoadOptions {
    /// Disables lazy-loading of membership events.
    Disabled,

    /// Enables lazy-loading of events.
    Enabled {
        /// If `true`, sends all membership events for all events, even if they have
        /// already been sent to the client. Defaults to `false`.
        include_redundant_members: bool,
    },
}

impl Serialize for LazyLoadOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state;
        match *self {
            Self::Enabled { include_redundant_members: true } => {
                state = serializer.serialize_struct("LazyLoad", 2)?;
                state.serialize_field("lazy_load_members", &true)?;
                state.serialize_field("include_redundant_members", &true)?;
            }
            Self::Enabled { .. } => {
                state = serializer.serialize_struct("LazyLoad", 1)?;
                state.serialize_field("lazy_load_members", &true)?;
            }
            _ => {
                state = serializer.serialize_struct("LazyLoad", 0)?;
            }
        }
        state.end()
    }
}

impl Default for LazyLoadOptions {
    fn default() -> Self {
        Self::Disabled
    }
}

struct LazyLoadOptionsVisitor;

impl<'de> Visitor<'de> for LazyLoadOptionsVisitor {
    type Value = LazyLoadOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Lazy load options")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut lazy_load_members = false;
        let mut include_redundant_members = false;
        while let Some((key, value)) = access.next_entry::<String, bool>()? {
            match &*key {
                "lazy_load_members" => lazy_load_members = value,
                "include_redundant_members" => include_redundant_members = value,
                _ => {}
            };
        }

        Ok(if lazy_load_members {
            LazyLoadOptions::Enabled { include_redundant_members }
        } else {
            LazyLoadOptions::Disabled
        })
    }
}

impl<'de> Deserialize<'de> for LazyLoadOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(LazyLoadOptionsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{LazyLoadOptions, UrlFilter};

    #[test]
    fn serialize_contains_url_filter_events_with_url() {
        let events_with_url = UrlFilter::EventsWithUrl;
        assert_eq!(to_json_value(events_with_url).unwrap(), json!(true))
    }

    #[test]
    fn serialize_contains_url_filter_events_without_url() {
        let events_without_url = UrlFilter::EventsWithoutUrl;
        assert_eq!(to_json_value(events_without_url).unwrap(), json!(false))
    }

    #[test]
    fn deserialize_contains_url_true() {
        let json = json!(true);
        assert_eq!(from_json_value::<UrlFilter>(json).unwrap(), UrlFilter::EventsWithUrl);
    }

    #[test]
    fn deserialize_contains_url_false() {
        let json = json!(false);
        assert_eq!(from_json_value::<UrlFilter>(json).unwrap(), UrlFilter::EventsWithoutUrl);
    }

    #[test]
    fn serialize_disabled_lazy_load() {
        let lazy_load_options = LazyLoadOptions::Disabled;
        assert_eq!(to_json_value(lazy_load_options).unwrap(), json!({}));
    }

    #[test]
    fn serialize_lazy_load_no_redundant() {
        let lazy_load_options = LazyLoadOptions::Enabled { include_redundant_members: false };
        assert_eq!(to_json_value(lazy_load_options).unwrap(), json!({ "lazy_load_members": true }));
    }

    #[test]
    fn serialize_lazy_load_with_redundant() {
        let lazy_load_options = LazyLoadOptions::Enabled { include_redundant_members: true };
        assert_eq!(
            to_json_value(lazy_load_options).unwrap(),
            json!({ "lazy_load_members": true, "include_redundant_members": true })
        );
    }

    #[test]
    fn deserialize_no_lazy_load() {
        let json = json!({});
        assert_eq!(from_json_value::<LazyLoadOptions>(json).unwrap(), LazyLoadOptions::Disabled,);
    }

    #[test]
    fn deserialize_ignore_redundant_members_when_no_lazy_load() {
        let json = json!({ "include_redundant_members": true });
        assert_eq!(from_json_value::<LazyLoadOptions>(json).unwrap(), LazyLoadOptions::Disabled,);
    }
}
