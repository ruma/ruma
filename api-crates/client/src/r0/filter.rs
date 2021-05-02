//! Endpoints for event filters.

pub mod create_filter;
pub mod get_filter;

mod lazy_load;
mod url;

pub use lazy_load::LazyLoadOptions;
pub use url::UrlFilter;

use js_int::UInt;
use ruma_identifiers::{RoomId, UserId};
use ruma_serde::{Outgoing, StringEnum};
use serde::Serialize;

/// Format to use for returned events.
#[derive(Clone, Debug, PartialEq, Eq, StringEnum)]
#[ruma_enum(rename_all = "snake_case")]
pub enum EventFormat {
    /// Client format, as described in the Client API.
    Client,

    /// Raw events from federation.
    Federation,

    #[doc(hidden)]
    _Custom(String),
}

impl Default for EventFormat {
    fn default() -> Self {
        Self::Client
    }
}

/// Filters to be applied to room events.
#[derive(Clone, Debug, Default, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(Clone, Default, Serialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rooms: Option<&'a [RoomId]>,

    /// A list of sender IDs to exclude.
    ///
    /// If this list is absent then no senders are excluded. A matching sender will be excluded
    /// even if it is listed in the 'senders' filter.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_senders: &'a [UserId],

    /// A list of senders IDs to include.
    ///
    /// If this list is absent then all senders are included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub senders: Option<&'a [UserId]>,

    /// A list of event types to include.
    ///
    /// If this list is absent then all event types are included. A '*' can be used as a wildcard
    /// to match any sequence of characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<&'a [String]>,

    /// Controls whether to include events with a URL key in their content.
    ///
    /// * `None`: No filtering
    /// * `Some(EventsWithUrl)`: Only events with a URL
    /// * `Some(EventsWithoutUrl)`: Only events without a URL
    #[serde(rename = "contains_url", skip_serializing_if = "Option::is_none")]
    pub url_filter: Option<UrlFilter>,

    /// Options to control lazy-loading of membership events.
    ///
    /// Defaults to `LazyLoadOptions::Disabled`.
    #[serde(flatten)]
    pub lazy_load_options: LazyLoadOptions,
}

impl<'a> RoomEventFilter<'a> {
    /// Creates an empty `RoomEventFilter`.
    ///
    /// You can also use the [`Default`] implementation.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a `RoomEventFilter` that can be used to ignore all room events.
    pub fn ignore_all() -> Self {
        Self { types: Some(&[]), ..Default::default() }
    }

    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        self.not_types.is_empty()
            && self.not_rooms.is_empty()
            && self.limit.is_none()
            && self.rooms.is_none()
            && self.not_senders.is_empty()
            && self.senders.is_none()
            && self.types.is_none()
            && self.url_filter.is_none()
            && self.lazy_load_options.is_disabled()
    }
}

impl IncomingRoomEventFilter {
    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        self.not_types.is_empty()
            && self.not_rooms.is_empty()
            && self.limit.is_none()
            && self.rooms.is_none()
            && self.not_senders.is_empty()
            && self.senders.is_none()
            && self.types.is_none()
            && self.url_filter.is_none()
            && self.lazy_load_options.is_disabled()
    }
}

/// Filters to be applied to room data.
#[derive(Clone, Debug, Default, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(Clone, Default, Serialize)]
pub struct RoomFilter<'a> {
    /// Include rooms that the user has left in the sync.
    ///
    /// Defaults to `false`.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub include_leave: bool,

    /// The per user account data to include for rooms.
    #[serde(default, skip_serializing_if = "ruma_serde::is_empty")]
    pub account_data: RoomEventFilter<'a>,

    /// The message and state update events to include for rooms.
    #[serde(default, skip_serializing_if = "ruma_serde::is_empty")]
    pub timeline: RoomEventFilter<'a>,

    /// The events that aren't recorded in the room history, e.g. typing and receipts, to include
    /// for rooms.
    #[serde(default, skip_serializing_if = "ruma_serde::is_empty")]
    pub ephemeral: RoomEventFilter<'a>,

    /// The state events to include for rooms.
    #[serde(default, skip_serializing_if = "ruma_serde::is_empty")]
    pub state: RoomEventFilter<'a>,

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rooms: Option<&'a [RoomId]>,
}

impl<'a> RoomFilter<'a> {
    /// Creates an empty `RoomFilter`.
    ///
    /// You can also use the [`Default`] implementation.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a `RoomFilter` that can be used to ignore all room events (of any type).
    pub fn ignore_all() -> Self {
        Self { rooms: Some(&[]), ..Default::default() }
    }

    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        !self.include_leave
            && self.account_data.is_empty()
            && self.timeline.is_empty()
            && self.ephemeral.is_empty()
            && self.state.is_empty()
            && self.not_rooms.is_empty()
            && self.rooms.is_none()
    }
}

impl IncomingRoomFilter {
    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        !self.include_leave
            && self.account_data.is_empty()
            && self.timeline.is_empty()
            && self.ephemeral.is_empty()
            && self.state.is_empty()
            && self.not_rooms.is_empty()
            && self.rooms.is_none()
    }
}

/// Filter for non-room data.
#[derive(Clone, Debug, Default, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(Clone, Default, Serialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub senders: Option<&'a [UserId]>,

    /// A list of event types to include.
    ///
    /// If this list is absent then all event types are included. A '*' can be used as a wildcard
    /// to match any sequence of characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<&'a [String]>,

    /// A list of sender IDs to exclude.
    ///
    /// If this list is absent then no senders are excluded. A matching sender will be excluded
    /// even if it is listed in the 'senders' filter.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub not_senders: &'a [UserId],
}

impl<'a> Filter<'a> {
    /// Creates an empty `Filter`.
    ///
    /// You can also use the [`Default`] implementation.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a `Filter` that can be used to ignore all events.
    pub fn ignore_all() -> Self {
        Self { types: Some(&[]), ..Default::default() }
    }

    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        self.not_types.is_empty()
            && self.limit.is_none()
            && self.senders.is_none()
            && self.types.is_none()
            && self.not_senders.is_empty()
    }
}

impl IncomingFilter {
    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        self.not_types.is_empty()
            && self.limit.is_none()
            && self.senders.is_none()
            && self.types.is_none()
            && self.not_senders.is_empty()
    }
}

/// A filter definition
#[derive(Clone, Debug, Default, Outgoing, Serialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[incoming_derive(Clone, Default, Serialize)]
pub struct FilterDefinition<'a> {
    /// List of event fields to include.
    ///
    /// If this list is absent then all fields are included. The entries may include '.' characters
    /// to indicate sub-fields. So ['content.body'] will include the 'body' field of the 'content'
    /// object. A literal '.' character in a field name may be escaped using a '\'. A server may
    /// include more fields than were requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_fields: Option<&'a [String]>,

    /// The format to use for events.
    ///
    /// 'client' will return the events in a format suitable for clients. 'federation' will return
    /// the raw event as received over federation. The default is 'client'.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub event_format: EventFormat,

    /// The presence updates to include.
    #[serde(default, skip_serializing_if = "ruma_serde::is_empty")]
    pub presence: Filter<'a>,

    /// The user account data that isn't associated with rooms to include.
    #[serde(default, skip_serializing_if = "ruma_serde::is_empty")]
    pub account_data: Filter<'a>,

    /// Filters to be applied to room data.
    #[serde(default, skip_serializing_if = "ruma_serde::is_empty")]
    pub room: RoomFilter<'a>,
}

impl<'a> FilterDefinition<'a> {
    /// Creates an empty `FilterDefinition`.
    ///
    /// You can also use the [`Default`] implementation.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a `FilterDefinition` that can be used to ignore all events.
    pub fn ignore_all() -> Self {
        Self {
            account_data: Filter::ignore_all(),
            room: RoomFilter::ignore_all(),
            presence: Filter::ignore_all(),
            ..Default::default()
        }
    }

    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        self.event_fields.is_none()
            && self.event_format == EventFormat::Client
            && self.presence.is_empty()
            && self.account_data.is_empty()
            && self.room.is_empty()
    }
}

impl IncomingFilterDefinition {
    /// Returns `true` if all fields are empty.
    pub fn is_empty(&self) -> bool {
        self.event_fields.is_none()
            && self.event_format == EventFormat::Client
            && self.presence.is_empty()
            && self.account_data.is_empty()
            && self.room.is_empty()
    }
}

macro_rules! can_be_empty {
    ($ty:ident $(<$gen:tt>)?) => {
        impl $(<$gen>)? ruma_serde::CanBeEmpty for $ty $(<$gen>)? {
            fn is_empty(&self) -> bool {
                self.is_empty()
            }
        }
    };
}

can_be_empty!(Filter<'a>);
can_be_empty!(FilterDefinition<'a>);
can_be_empty!(RoomEventFilter<'a>);
can_be_empty!(RoomFilter<'a>);

can_be_empty!(IncomingFilter);
can_be_empty!(IncomingFilterDefinition);
can_be_empty!(IncomingRoomEventFilter);
can_be_empty!(IncomingRoomFilter);

#[cfg(test)]
mod tests {
    use matches::assert_matches;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{
        Filter, FilterDefinition, IncomingFilterDefinition, IncomingRoomEventFilter,
        IncomingRoomFilter, LazyLoadOptions, RoomEventFilter, RoomFilter, UrlFilter,
    };

    #[test]
    fn default_filters_are_empty() -> Result<(), serde_json::Error> {
        assert_eq!(to_json_value(Filter::default())?, json!({}));
        assert_eq!(to_json_value(FilterDefinition::default())?, json!({}));
        assert_eq!(to_json_value(RoomEventFilter::default())?, json!({}));
        assert_eq!(to_json_value(RoomFilter::default())?, json!({}));

        Ok(())
    }

    #[test]
    fn filter_definition_roundtrip() -> Result<(), serde_json::Error> {
        let filter = FilterDefinition::default();
        let filter_str = to_json_value(&filter)?;

        let incoming_filter = from_json_value::<IncomingFilterDefinition>(filter_str)?;
        assert!(incoming_filter.is_empty());

        Ok(())
    }

    #[test]
    fn room_filter_definition_roundtrip() -> Result<(), serde_json::Error> {
        let filter = RoomFilter::default();
        let room_filter = to_json_value(&filter)?;

        let incoming_room_filter = from_json_value::<IncomingRoomFilter>(room_filter)?;
        assert!(incoming_room_filter.is_empty());

        Ok(())
    }

    #[test]
    fn issue_366() -> Result<(), serde_json::Error> {
        let obj = json!({
            "lazy_load_members": true,
            "filter_json": { "contains_url": true, "types": ["m.room.message"] },
            "types": ["m.room.message"],
            "not_types": [],
            "rooms": null,
            "not_rooms": [],
            "senders": null,
            "not_senders": [],
            "contains_url": true,
        });

        assert_matches!(
            from_json_value(obj)?,
            IncomingRoomEventFilter {
                types: Some(types),
                not_types,
                rooms: None,
                not_rooms,
                senders: None,
                not_senders,
                limit: None,
                url_filter: Some(UrlFilter::EventsWithUrl),
                lazy_load_options: LazyLoadOptions::Enabled { include_redundant_members: false },
            } if types == vec!["m.room.message".to_owned()]
                && not_types.is_empty()
                && not_rooms.is_empty()
                && not_senders.is_empty()
        );

        Ok(())
    }
}
