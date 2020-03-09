//! Endpoints for event filters.

pub mod create_filter;
pub mod get_filter;

use std::fmt;

use js_int::UInt;
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

/// Filters to be applied to room events
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RoomEventFilter {
    /// A list of event types to exclude.
    ///
    /// If this list is absent then no event types are excluded. A matching type will be excluded
    /// even if it is listed in the 'types' filter. A '*' can be used as a wildcard to match any
    /// sequence of characters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_types: Vec<String>,
    /// A list of room IDs to exclude.
    ///
    /// If this list is absent then no rooms are excluded. A matching room will be excluded even if
    /// it is listed in the 'rooms' filter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_rooms: Vec<String>,
    /// The maximum number of events to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<UInt>,
    /// A list of room IDs to include.
    ///
    /// If this list is absent then all rooms are included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<RoomId>>,
    /// A list of sender IDs to exclude.
    ///
    /// If this list is absent then no senders are excluded. A matching sender will be excluded even
    /// if it is listed in the 'senders' filter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_senders: Vec<UserId>,
    /// A list of senders IDs to include.
    ///
    /// If this list is absent then all senders are included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub senders: Option<Vec<UserId>>,
    /// A list of event types to include.
    ///
    /// If this list is absent then all event types are included. A '*' can be used as a wildcard to
    /// match any sequence of characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
    /// If `true` include only events with a URL key in their content.
    /// If `false`, exclude such events.
    ///
    /// If this item is absent then all event types are included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains_url: Option<bool>,
    /// Options to control lazy-loading of membership events.
    #[serde(flatten)]
    pub lazy_load_options: LazyLoadOptions,
}

impl RoomEventFilter {
    /// A filter that can be used to ignore all room events
    pub fn ignore_all() -> Self {
        Self {
            types: Some(vec![]),
            ..Default::default()
        }
    }
}

/// Filters to be applied to room data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RoomFilter {
    /// Include rooms that the user has left in the sync.
    ///
    /// Defaults to false if not included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_leave: Option<bool>,
    /// The per user account data to include for rooms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_data: Option<RoomEventFilter>,
    /// The message and state update events to include for rooms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline: Option<RoomEventFilter>,
    /// The events that aren't recorded in the room history, e.g. typing and receipts, to include
    /// for rooms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<RoomEventFilter>,
    /// The state events to include for rooms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<RoomEventFilter>,
    /// A list of room IDs to exclude.
    ///
    /// If this list is absent then no rooms are excluded. A matching room will be excluded even if
    /// it is listed in the 'rooms' filter. This filter is applied before the filters in
    /// `ephemeral`, `state`, `timeline` or `account_data`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_rooms: Vec<RoomId>,
    /// A list of room IDs to include.
    ///
    /// If this list is absent then all rooms are included. This filter is applied before the
    /// filters in `ephemeral`, `state`, `timeline` or `account_data`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rooms: Option<Vec<RoomId>>,
}

impl RoomFilter {
    /// A filter that can be used to ignore all room events (of any type)
    pub fn ignore_all() -> Self {
        Self {
            rooms: Some(vec![]),
            ..Default::default()
        }
    }
}

/// Filter for not-room data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Filter {
    /// A list of event types to exclude.
    ///
    /// If this list is absent then no event types are excluded. A matching type will be excluded
    /// even if it is listed in the 'types' filter. A '*' can be used as a wildcard to match any
    /// sequence of characters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_types: Vec<String>,
    /// The maximum number of events to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<UInt>,
    /// A list of senders IDs to include.
    ///
    /// If this list is absent then all senders are included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub senders: Option<Vec<UserId>>,
    /// A list of event types to include.
    ///
    /// If this list is absent then all event types are included. A '*' can be used as a wildcard to
    /// match any sequence of characters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
    /// A list of sender IDs to exclude.
    ///
    /// If this list is absent then no senders are excluded. A matching sender will be excluded even
    /// if it is listed in the 'senders' filter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_senders: Vec<UserId>,
}

impl Filter {
    /// A filter that can be used to ignore all events
    pub fn ignore_all() -> Self {
        Self {
            types: Some(vec![]),
            ..Default::default()
        }
    }
}

/// A filter definition
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FilterDefinition {
    /// List of event fields to include.
    ///
    /// If this list is absent then all fields are included. The entries may include '.' charaters
    /// to indicate sub-fields. So ['content.body'] will include the 'body' field of the 'content'
    /// object. A literal '.' character in a field name may be escaped using a '\'. A server may
    /// include more fields than were requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_fields: Option<Vec<String>>,
    /// The format to use for events.
    ///
    /// 'client' will return the events in a format suitable for clients. 'federation' will return
    /// the raw event as received over federation. The default is 'client'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_format: Option<EventFormat>,
    /// The user account data that isn't associated with rooms to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_data: Option<Filter>,
    /// Filters to be applied to room data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room: Option<RoomFilter>,
    /// The presence updates to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Filter>,
}

impl FilterDefinition {
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
            Self::Enabled {
                include_redundant_members: true,
            } => {
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
            LazyLoadOptions::Enabled {
                include_redundant_members,
            }
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
    use serde_json::json;

    use super::LazyLoadOptions;

    #[test]
    fn test_serializing_disabled_lazy_load() {
        let lazy_load_options = LazyLoadOptions::Disabled;
        assert_eq!(serde_json::to_value(lazy_load_options).unwrap(), json!({}));
    }

    #[test]
    fn test_serializing_lazy_load_no_redundant() {
        let lazy_load_options = LazyLoadOptions::Enabled {
            include_redundant_members: false,
        };
        assert_eq!(
            serde_json::to_value(lazy_load_options).unwrap(),
            json!({ "lazy_load_members": true })
        );
    }

    #[test]
    fn test_serializing_lazy_load_with_redundant() {
        let lazy_load_options = LazyLoadOptions::Enabled {
            include_redundant_members: true,
        };
        assert_eq!(
            serde_json::to_value(lazy_load_options).unwrap(),
            json!({ "lazy_load_members": true, "include_redundant_members": true })
        );
    }

    #[test]
    fn test_deserializing_no_lazy_load() {
        let json = json!({});
        assert_eq!(
            serde_json::from_value::<LazyLoadOptions>(json).unwrap(),
            LazyLoadOptions::Disabled,
        );
    }

    #[test]
    fn test_deserializing_ignore_redundant_members_when_no_lazy_load() {
        let json = json!({ "include_redundant_members": true });
        assert_eq!(
            serde_json::from_value::<LazyLoadOptions>(json).unwrap(),
            LazyLoadOptions::Disabled,
        );
    }
}
