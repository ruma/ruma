//! `POST /_matrix/client/*/search`
//!
//! Search events.

mod result_group_map_serde;

pub mod v3 {
    //! `/v3/` ([spec])
    //!
    //! [spec]: https://spec.matrix.org/v1.18/client-server-api/#post_matrixclientv3search

    use std::{
        collections::{BTreeMap, btree_map},
        ops::Deref,
    };

    use as_variant::as_variant;
    use js_int::{UInt, uint};
    use ruma_common::{
        OwnedEventId, OwnedMxcUri, OwnedRoomId, OwnedUserId,
        api::{auth_scheme::AccessToken, request, response},
        metadata,
        serde::{Raw, StringEnum},
    };
    use ruma_events::{AnyStateEvent, AnyTimelineEvent};
    use serde::{Deserialize, Serialize};

    use crate::{PrivOwnedStr, filter::RoomEventFilter};

    metadata! {
        method: POST,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            1.0 => "/_matrix/client/r0/search",
            1.1 => "/_matrix/client/v3/search",
        }
    }

    /// Request type for the `search` endpoint.
    #[request(error = crate::Error)]
    pub struct Request {
        /// The point to return events from.
        ///
        /// If given, this should be a `next_batch` result from a previous call to this endpoint.
        #[ruma_api(query)]
        pub next_batch: Option<String>,

        /// Describes which categories to search in and their criteria.
        pub search_categories: Categories,
    }

    /// Response type for the `search` endpoint.
    #[response(error = crate::Error)]
    pub struct Response {
        /// A grouping of search results by category.
        pub search_categories: ResultCategories,
    }

    impl Request {
        /// Creates a new `Request` with the given categories.
        pub fn new(search_categories: Categories) -> Self {
            Self { next_batch: None, search_categories }
        }
    }

    impl Response {
        /// Creates a new `Response` with the given search results.
        pub fn new(search_categories: ResultCategories) -> Self {
            Self { search_categories }
        }
    }

    /// Categories of events that can be searched for.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Categories {
        /// Criteria for searching room events.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_events: Option<Criteria>,
    }

    impl Categories {
        /// Creates an empty `Categories`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// Criteria for searching a category of events.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Criteria {
        /// The string to search events for.
        pub search_term: String,

        /// The keys to search for.
        ///
        /// Defaults to all keys.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub keys: Option<Vec<SearchKeys>>,

        /// A `Filter` to apply to the search.
        #[serde(default, skip_serializing_if = "RoomEventFilter::is_empty")]
        pub filter: RoomEventFilter,

        /// The order in which to search for results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub order_by: Option<OrderBy>,

        /// Configures whether any context for the events returned are included in the response.
        #[serde(default, skip_serializing_if = "EventContext::is_default")]
        pub event_context: EventContext,

        /// Requests the server return the current state for each room returned.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub include_state: Option<bool>,

        /// Requests that the server partitions the result set based on the provided list of keys.
        #[serde(default, skip_serializing_if = "Groupings::is_empty")]
        pub groupings: Groupings,
    }

    impl Criteria {
        /// Creates a new `Criteria` with the given search term.
        pub fn new(search_term: String) -> Self {
            Self {
                search_term,
                keys: None,
                filter: RoomEventFilter::default(),
                order_by: None,
                event_context: Default::default(),
                include_state: None,
                groupings: Default::default(),
            }
        }
    }

    /// Configures whether any context for the events returned are included in the response.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
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
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub include_profile: bool,
    }

    fn default_event_context_limit() -> UInt {
        uint!(5)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default_event_context_limit(val: &UInt) -> bool {
        *val == default_event_context_limit()
    }

    impl EventContext {
        /// Creates an `EventContext` with all-default values.
        pub fn new() -> Self {
            Self {
                before_limit: default_event_context_limit(),
                after_limit: default_event_context_limit(),
                include_profile: false,
            }
        }

        /// Returns whether all fields have their default value.
        pub fn is_default(&self) -> bool {
            self.before_limit == default_event_context_limit()
                && self.after_limit == default_event_context_limit()
                && !self.include_profile
        }
    }

    impl Default for EventContext {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Context for search results, if requested.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct EventContextResult {
        /// Pagination token for the end of the chunk.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub end: Option<String>,

        /// Events just after the result.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub events_after: Vec<Raw<AnyTimelineEvent>>,

        /// Events just before the result.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub events_before: Vec<Raw<AnyTimelineEvent>>,

        /// The historic profile information of the users that sent the events returned.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub profile_info: BTreeMap<OwnedUserId, UserProfile>,

        /// Pagination token for the start of the chunk.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub start: Option<String>,
    }

    impl EventContextResult {
        /// Creates an empty `EventContextResult`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns whether all fields are `None` or an empty list.
        pub fn is_empty(&self) -> bool {
            self.end.is_none()
                && self.events_after.is_empty()
                && self.events_before.is_empty()
                && self.profile_info.is_empty()
                && self.start.is_none()
        }
    }

    /// A grouping for partitioning the result set.
    #[derive(Clone, Default, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Grouping {
        /// The key within events to use for this grouping.
        pub key: Option<GroupingKey>,
    }

    impl Grouping {
        /// Creates an empty `Grouping`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns whether `key` is `None`.
        pub fn is_empty(&self) -> bool {
            self.key.is_none()
        }
    }

    /// The key within events to use for this grouping.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum)]
    #[ruma_enum(rename_all = "snake_case")]
    #[non_exhaustive]
    pub enum GroupingKey {
        /// `room_id`
        RoomId,

        /// `sender`
        Sender,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// Requests that the server partitions the result set based on the provided list of keys.
    #[derive(Clone, Default, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Groupings {
        /// List of groups to request.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub group_by: Vec<Grouping>,
    }

    impl Groupings {
        /// Creates an empty `Groupings`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns `true` if all fields are empty.
        pub fn is_empty(&self) -> bool {
            self.group_by.is_empty()
        }
    }

    /// The keys to search for.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum)]
    #[non_exhaustive]
    pub enum SearchKeys {
        /// content.body
        #[ruma_enum(rename = "content.body")]
        ContentBody,

        /// content.name
        #[ruma_enum(rename = "content.name")]
        ContentName,

        /// content.topic
        #[ruma_enum(rename = "content.topic")]
        ContentTopic,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// The order in which to search for results.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[ruma_enum(rename_all = "snake_case")]
    pub enum OrderBy {
        /// Prioritize recent events.
        Recent,

        /// Prioritize events by a numerical ranking of how closely they matched the search
        /// criteria.
        Rank,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    /// Categories of events that can be searched for.
    #[derive(Clone, Default, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ResultCategories {
        /// Room event results.
        #[serde(default, skip_serializing_if = "ResultRoomEvents::is_empty")]
        pub room_events: ResultRoomEvents,
    }

    impl ResultCategories {
        /// Creates an empty `ResultCategories`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// Categories of events that can be searched for.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ResultRoomEvents {
        /// An approximate count of the total number of results found.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub count: Option<UInt>,

        /// Any groups that were requested.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub groups: ResultGroupMapsByGroupingKey,

        /// Token that can be used to get the next batch of results, by passing as the `next_batch`
        /// parameter to the next call.
        ///
        /// If this field is absent, there are no more results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,

        /// List of results in the requested order.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub results: Vec<SearchResult>,

        /// The current state for every room in the results.
        ///
        /// This is included if the request had the `include_state` key set with a value of `true`.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub state: BTreeMap<OwnedRoomId, Vec<Raw<AnyStateEvent>>>,

        /// List of words which should be highlighted, useful for stemming which may
        /// change the query terms.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub highlights: Vec<String>,
    }

    impl ResultRoomEvents {
        /// Creates an empty `ResultRoomEvents`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns `true` if all fields are empty / `None`.
        pub fn is_empty(&self) -> bool {
            self.count.is_none()
                && self.groups.is_empty()
                && self.next_batch.is_none()
                && self.results.is_empty()
                && self.state.is_empty()
                && self.highlights.is_empty()
        }
    }

    /// A map of [`GroupingKey`] to the associated [`ResultGroupMap`].
    ///
    /// This type is used to ensure that a supported [`ResultGroupMap`] always uses the appropriate
    /// [`GroupingKey`].
    #[derive(Clone, Debug, Default)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ResultGroupMapsByGroupingKey(BTreeMap<GroupingKey, ResultGroupMap>);

    impl ResultGroupMapsByGroupingKey {
        /// Construct an empty `ResultGroupMapsByGroupingKey`.
        pub fn new() -> Self {
            Self::default()
        }

        /// Insert the given [`ResultGroupMap`].
        ///
        /// If a map with the same [`GroupingKey`] was already present, it is returned.
        pub fn insert(&mut self, map: ResultGroupMap) -> Option<ResultGroupMap> {
            self.0.insert(map.grouping_key(), map)
        }
    }

    impl Deref for ResultGroupMapsByGroupingKey {
        type Target = BTreeMap<GroupingKey, ResultGroupMap>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl FromIterator<ResultGroupMap> for ResultGroupMapsByGroupingKey {
        fn from_iter<T: IntoIterator<Item = ResultGroupMap>>(iter: T) -> Self {
            Self(iter.into_iter().map(|map| (map.grouping_key(), map)).collect())
        }
    }

    impl Extend<ResultGroupMap> for ResultGroupMapsByGroupingKey {
        fn extend<T: IntoIterator<Item = ResultGroupMap>>(&mut self, iter: T) {
            self.0.extend(iter.into_iter().map(|map| (map.grouping_key(), map)));
        }
    }

    impl IntoIterator for ResultGroupMapsByGroupingKey {
        type Item = ResultGroupMap;
        type IntoIter = btree_map::IntoValues<GroupingKey, ResultGroupMap>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_values()
        }
    }

    /// A map of results grouped by key.
    #[derive(Clone, Debug)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum ResultGroupMap {
        /// Results grouped by room ID.
        RoomId(BTreeMap<OwnedRoomId, ResultGroup>),

        /// Results grouped by sender.
        Sender(BTreeMap<OwnedUserId, ResultGroup>),

        #[doc(hidden)]
        _Custom(CustomResultGroupMap),
    }

    impl ResultGroupMap {
        /// The key that was used to group this map.
        pub fn grouping_key(&self) -> GroupingKey {
            match self {
                Self::RoomId(_) => GroupingKey::RoomId,
                Self::Sender(_) => GroupingKey::Sender,
                Self::_Custom(custom) => custom.grouping_key.as_str().into(),
            }
        }

        /// The map of grouped results, if this uses a custom key.
        pub fn custom_map(&self) -> Option<&BTreeMap<String, ResultGroup>> {
            as_variant!(self, Self::_Custom).map(|custom| &custom.map)
        }

        /// Convert this into the map of grouped results, if this uses a custom key.
        pub fn into_custom_map(self) -> Option<BTreeMap<String, ResultGroup>> {
            as_variant!(self, Self::_Custom).map(|custom| custom.map)
        }
    }

    /// A map of results grouped by custom key type.
    #[doc(hidden)]
    #[derive(Clone, Debug)]
    pub struct CustomResultGroupMap {
        /// The type of key that was used to group the results.
        pub(super) grouping_key: String,

        /// The grouped results.
        pub(super) map: BTreeMap<String, ResultGroup>,
    }

    /// A group of results.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ResultGroup {
        /// Token that can be used to get the next batch of results in the group, by passing as the
        /// `next_batch` parameter to the next call.
        ///
        /// If this field is absent, there are no more results in this group.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,

        /// Key that can be used to order different groups.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub order: Option<UInt>,

        /// Which results are in this group.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub results: Vec<OwnedEventId>,
    }

    impl ResultGroup {
        /// Creates an empty `ResultGroup`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns `true` if all fields are empty / `None`.
        pub fn is_empty(&self) -> bool {
            self.next_batch.is_none() && self.order.is_none() && self.results.is_empty()
        }
    }

    /// A search result.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct SearchResult {
        /// Context for result, if requested.
        #[serde(default, skip_serializing_if = "EventContextResult::is_empty")]
        pub context: EventContextResult,

        /// A number that describes how closely this result matches the search.
        ///
        /// Higher is closer.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rank: Option<f64>,

        /// The event that matched.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result: Option<Raw<AnyTimelineEvent>>,
    }

    impl SearchResult {
        /// Creates an empty `SearchResult`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns `true` if all fields are empty / `None`.
        pub fn is_empty(&self) -> bool {
            self.context.is_empty() && self.rank.is_none() && self.result.is_none()
        }
    }

    /// A user profile.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct UserProfile {
        /// The user's avatar URL, if set.
        ///
        /// If you activate the `compat-empty-string-null` feature, this field being an empty
        /// string in JSON will result in `None` here during deserialization.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[cfg_attr(
            feature = "compat-empty-string-null",
            serde(default, deserialize_with = "ruma_common::serde::empty_string_as_none")
        )]
        pub avatar_url: Option<OwnedMxcUri>,

        /// The user's display name, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,
    }

    impl UserProfile {
        /// Creates an empty `UserProfile`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns `true` if all fields are `None`.
        pub fn is_empty(&self) -> bool {
            self.avatar_url.is_none() && self.displayname.is_none()
        }
    }
}

#[cfg(all(test, feature = "client", feature = "server"))]
mod tests {
    use std::{borrow::Cow, collections::BTreeMap};

    use assert_matches2::assert_matches;
    use js_int::uint;
    use ruma_common::{
        api::{
            IncomingRequest, IncomingResponse, OutgoingRequest, OutgoingResponse,
            SupportedVersions, auth_scheme::SendAccessToken,
        },
        event_id, room_id,
    };
    use serde_json::{
        Value as JsonValue, from_slice as from_json_slice, json, to_vec as to_json_vec,
    };

    use super::v3::{GroupingKey, OrderBy, Request, Response, ResultGroupMap, SearchKeys};

    #[test]
    fn request_roundtrip() {
        let body = json!({
            "search_categories": {
                "room_events": {
                    "groupings": {
                        "group_by": [
                            { "key": "room_id" },
                        ],
                    },
                    "keys": ["content.body"],
                    "order_by": "recent",
                    "search_term": "martians and men"
                }
            }
        });

        let http_request = http::Request::post("http://localhost/_matrix/client/v3/search")
            .body(to_json_vec(&body).unwrap())
            .unwrap();
        let request = Request::try_from_http_request(http_request, &[] as &[&str]).unwrap();

        let criteria = request.search_categories.room_events.as_ref().unwrap();
        assert_eq!(criteria.groupings.group_by.len(), 1);
        assert_eq!(criteria.groupings.group_by[0].key, Some(GroupingKey::RoomId));
        let keys = criteria.keys.as_ref().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], SearchKeys::ContentBody);
        assert_eq!(criteria.order_by, Some(OrderBy::Recent));
        assert_eq!(criteria.search_term, "martians and men");

        let http_request = request
            .try_into_http_request::<Vec<u8>>(
                "http://localhost",
                SendAccessToken::IfRequired("access_token"),
                Cow::Owned(SupportedVersions::from_parts(&["v1.4".to_owned()], &BTreeMap::new())),
            )
            .unwrap();
        assert_eq!(from_json_slice::<JsonValue>(http_request.body()).unwrap(), body);
    }

    #[test]
    fn response_roundtrip() {
        let body = json!({
            "search_categories": {
                "room_events": {
                    "count": 1224,
                    "groups": {
                        "room_id": {
                            "!qPewotXpIctQySfjSy:localhost": {
                                "next_batch": "BdgFsdfHSf-dsFD",
                                "order": 1,
                                "results": ["$144429830826TWwbB:localhost"],
                            },
                        },
                    },
                    "highlights": [
                        "martians",
                        "men",
                    ],
                    "next_batch": "5FdgFsd234dfgsdfFD",
                    "results": [
                        {
                            "rank": 0.004_248_66,
                            "result": {
                                "content": {
                                    "body": "This is an example text message",
                                    "format": "org.matrix.custom.html",
                                    "formatted_body": "<b>This is an example text message</b>",
                                    "msgtype": "m.text",
                                },
                                "event_id": "$144429830826TWwbB:localhost",
                                "origin_server_ts": 1_735_824_653,
                                "room_id": "!qPewotXpIctQySfjSy:localhost",
                                "sender": "@example:example.org",
                                "type": "m.room.message",
                                "unsigned": {
                                    "age": 1234,
                                    "membership": "join",
                                }
                            }
                        }
                    ]
                }
            }
        });
        let result_event_id = event_id!("$144429830826TWwbB:localhost");

        let http_request = http::Response::new(to_json_vec(&body).unwrap());
        let response = Response::try_from_http_response(http_request).unwrap();

        let results = &response.search_categories.room_events;
        assert_eq!(results.count, Some(uint!(1224)));
        assert_eq!(results.groups.len(), 1);
        assert_matches!(
            results.groups.get(&GroupingKey::RoomId),
            Some(ResultGroupMap::RoomId(room_id_group_map))
        );
        assert_eq!(room_id_group_map.len(), 1);
        let room_id_group =
            room_id_group_map.get(room_id!("!qPewotXpIctQySfjSy:localhost")).unwrap();
        assert_eq!(room_id_group.results, &[result_event_id]);
        assert_eq!(results.highlights, &["martians", "men"]);
        assert_eq!(results.next_batch.as_deref(), Some("5FdgFsd234dfgsdfFD"));
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.results[0].rank, Some(0.004_248_66));
        let result = results.results[0].result.as_ref().unwrap().deserialize().unwrap();
        assert_eq!(result.event_id(), result_event_id);

        let http_response = response.try_into_http_response::<Vec<u8>>().unwrap();
        assert_eq!(from_json_slice::<JsonValue>(http_response.body()).unwrap(), body);
    }
}
