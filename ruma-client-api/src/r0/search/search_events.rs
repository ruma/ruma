//! [POST /_matrix/client/r0/search](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-search)

use std::collections::BTreeMap;

use js_int::{uint, UInt};
use ruma_api::ruma_api;
use ruma_events::{AnyEvent, AnyStateEvent, EventJson};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};

use crate::r0::filter::RoomEventFilter;

ruma_api! {
    metadata: {
        description: "Search events.",
        method: POST,
        name: "search",
        path: "/_matrix/client/r0/search",
        rate_limited: true,
        requires_authentication: true,
    }

    request: {
        /// The point to return events from.
        ///
        /// If given, this should be a `next_batch` result from a previous call to this endpoint.
        #[ruma_api(query)]
        pub next_batch: Option<String>,

        /// Describes which categories to search in and their criteria.
        pub search_categories: Categories,
    }

    response: {
        /// A grouping of search results by category.
        pub search_categories: ResultCategories,
    }

    error: crate::Error
}

/// Categories of events that can be searched for.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Categories {
    /// Criteria for searching a category of events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_events: Option<Criteria>,
}

/// Criteria for searching a category of events.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Criteria {
    /// The string to search events for.
    pub search_term: String,

    /// The keys to search for. Defaults to all keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<SearchKeys>>,

    /// A `Filter` to apply to the search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<RoomEventFilter>,

    /// The order in which to search for results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<OrderBy>,

    /// Configures whether any context for the events returned are included in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_context: Option<EventContext>,

    /// Requests the server return the current state for each room returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_state: Option<bool>,

    /// Requests that the server partitions the result set based on the provided list of keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groupings: Option<Groupings>,
}

/// Configures whether any context for the events returned are included in the response.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct EventContext {
    /// How many events before the result are returned.
    #[serde(
        default = "default_event_context_limit",
        skip_serializing_if = "is_default_event_context_limit"
    )]
    pub before_limit: UInt,

    /// How many events after the result are returned.
    #[serde(
        default = "default_event_context_limit",
        skip_serializing_if = "is_default_event_context_limit"
    )]
    pub after_limit: UInt,

    /// Requests that the server returns the historic profile information for the users that
    /// sent the events that were returned.
    #[serde(default, skip_serializing_if = "ruma_serde::is_default")]
    pub include_profile: bool,
}

fn default_event_context_limit() -> UInt {
    uint!(5)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_default_event_context_limit(val: &UInt) -> bool {
    *val == default_event_context_limit()
}

/// Context for search results, if requested.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventContextResult {
    /// Pagination token for the end of the chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,

    /// Events just after the result.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events_after: Vec<EventJson<AnyEvent>>,

    /// Events just before the result.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events_before: Vec<EventJson<AnyEvent>>,

    /// The historic profile information of the users that sent the events returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_info: Option<BTreeMap<UserId, UserProfile>>,

    /// Pagination token for the start of the chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
}

/// A grouping for partioning the result set.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Grouping {
    /// The key within events to use for this grouping.
    pub key: Option<GroupingKey>,
}

/// The key within events to use for this grouping.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupingKey {
    /// `room_id`
    RoomId,

    /// `sender`
    Sender,
}

/// Requests that the server partitions the result set based on the provided list of keys.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Groupings {
    /// List of groups to request.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_by: Vec<Grouping>,
}

/// The keys to search for.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SearchKeys {
    /// content.body
    #[serde(rename = "content.body")]
    ContentBody,

    /// content.name
    #[serde(rename = "content.name")]
    ContentName,

    /// content.topic
    #[serde(rename = "content.topic")]
    ContentTopic,
}

/// The order in which to search for results.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderBy {
    /// Prioritize recent events.
    Recent,

    /// Prioritize events by a numerical ranking of how closely they matched the search
    /// criteria.
    Rank,
}

/// Categories of events that can be searched for.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResultCategories {
    /// Room event results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_events: Option<RoomEventJsons>,
}

/// Categories of events that can be searched for.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoomEventJsons {
    /// An approximate count of the total number of results found.
    pub count: UInt,

    /// Any groups that were requested.
    pub groups: BTreeMap<GroupingKey, BTreeMap<RoomIdOrUserId, ResultGroup>>,

    /// Token that can be used to get the next batch of results, by passing as the `next_batch`
    /// parameter to the next call. If this field is absent, there are no more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_batch: Option<String>,

    /// List of results in the requested order.
    pub results: Vec<SearchResult>,

    /// The current state for every room in the results. This is included if the request had the
    /// `include_state` key set with a value of `true`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state: BTreeMap<RoomId, Vec<EventJson<AnyStateEvent>>>,

    /// List of words which should be highlighted, useful for stemming which may
    /// change the query terms.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub highlights: Vec<String>,
}

/// A grouping of results, if requested.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResultGroup {
    /// Token that can be used to get the next batch of results in the group, by passing as the
    /// `next_batch` parameter to the next call. If this field is absent, there are no more
    /// results in this group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_batch: Option<String>,

    /// Key that can be used to order different groups.
    pub order: UInt,

    /// Which results are in this group.
    pub results: Vec<EventId>,
}

/// A search result.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SearchResult {
    /// Context for result, if requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<EventContextResult>,

    /// A number that describes how closely this result matches the search. Higher is closer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<UInt>,

    /// The event that matched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<EventJson<AnyEvent>>,
}

/// A user profile.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UserProfile {
    /// The user's avatar URL, if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    /// The user's display name, if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayname: Option<String>,
}

/// Represents either a room or user ID for returning grouped search results.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum RoomIdOrUserId {
    /// Represents a room ID.
    RoomId(RoomId),

    /// Represents a user ID.
    UserId(UserId),
}
