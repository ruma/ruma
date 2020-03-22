//! [POST /_matrix/client/r0/search](https://matrix.org/docs/spec/client_server/r0.4.0.html#post-matrix-client-r0-search)

use std::collections::HashMap;

use js_int::UInt;
use ruma_api::{ruma_api, Outgoing};
use ruma_events::{collections::all::Event, EventResult};
use ruma_identifiers::{EventId, RoomId, UserId};
use serde::{Deserialize, Serialize};

use crate::r0::filter::RoomEventFilter;

ruma_api! {
    metadata {
        description: "Search events.",
        method: POST,
        name: "search",
        path: "/_matrix/client/r0/search",
        rate_limited: true,
        requires_authentication: true,
    }

    request {
        /// The point to return events from.
        ///
        /// If given, this should be a `next_batch` result from a previous call to this endpoint.
        #[ruma_api(query)]
        pub next_batch: Option<String>,
        /// Describes which categories to search in and their criteria.
        pub search_categories: Categories,
    }

    response {
        /// A grouping of search results by category.
        #[wrap_incoming]
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
    /// Configures whether any context for the events returned are included in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_context: Option<EventContext>,
    /// A `Filter` to apply to the search.
    // TODO: "timeline" key might need to be included.
    // See https://github.com/matrix-org/matrix-doc/issues/598.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<RoomEventFilter>,
    /// Requests that the server partitions the result set based on the provided list of keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groupings: Option<Groupings>,
    /// Requests the server return the current state for each room returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_state: Option<bool>,
    /// The keys to search for. Defaults to all keys.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<SearchKeys>,
    /// The order in which to search for results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<OrderBy>,
    /// The string to search events for.
    pub search_term: String,
}

/// Configures whether any context for the events returned are included in the response.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct EventContext {
    /// How many events after the result are returned.
    pub after_limit: UInt,
    /// How many events before the result are returned.
    pub before_limit: UInt,
    /// Requests that the server returns the historic profile information for the users that
    /// sent the events that were returned.
    pub include_profile: bool,
}

/// Context for search results, if requested.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct EventContextResult {
    /// Pagination token for the end of the chunk.
    pub end: String,
    /// Events just after the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[wrap_incoming(Event with EventResult)]
    pub events_after: Option<Vec<Event>>,
    /// Events just before the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[wrap_incoming(Event with EventResult)]
    pub events_before: Option<Vec<Event>>,
    /// The historic profile information of the users that sent the events returned.
    // TODO: Not sure this is right. https://github.com/matrix-org/matrix-doc/issues/773
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_info: Option<HashMap<UserId, UserProfile>>,
    /// Pagination token for the start of the chunk.
    pub start: String,
}

/// A grouping for partioning the result set.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Grouping {
    /// The key within events to use for this grouping.
    pub key: GroupingKey,
}

/// The key within events to use for this grouping.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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
    /// Prioritize events by a numerical ranking of how closely they matched the search
    /// criteria.
    Rank,
    /// Prioritize recent events.
    Recent,
}

/// Categories of events that can be searched for.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct ResultCategories {
    /// Room event results.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[wrap_incoming(RoomEventResults)]
    pub room_events: Option<RoomEventResults>,
}

/// Categories of events that can be searched for.
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct RoomEventResults {
    /// An approximate count of the total number of results found.
    pub count: UInt,
    /// Any groups that were requested.
    // TODO: Not sure this is right. https://github.com/matrix-org/matrix-doc/issues/773
    pub groups: HashMap<GroupingKey, HashMap<RoomId, ResultGroup>>,
    /// Token that can be used to get the next batch of results, by passing as the `next_batch`
    /// parameter to the next call. If this field is absent, there are no more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_batch: Option<String>,
    /// List of results in the requested order.
    #[wrap_incoming(SearchResult)]
    pub results: Vec<SearchResult>,
    /// The current state for every room in the results. This is included if the request had the
    /// `include_state` key set with a value of `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    // TODO: Major WTF here. https://github.com/matrix-org/matrix-doc/issues/773
    pub state: Option<()>,
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
#[derive(Clone, Debug, Serialize, Outgoing)]
pub struct SearchResult {
    /// Context for result, if requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[wrap_incoming(EventContextResult)]
    pub context: Option<EventContextResult>,
    /// A number that describes how closely this result matches the search. Higher is closer.
    pub rank: f64,
    /// The event that matched.
    #[wrap_incoming(with EventResult)]
    pub result: Event,
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
