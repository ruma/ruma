//! `POST /_matrix/client/v4/sync` ([MSC])
//!
//! Get all new events in a sliding window of rooms since the last sync or a given point in time.
//!
//! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4186

use std::{collections::BTreeMap, time::Duration};

use js_int::UInt;
use js_option::JsOption;
use ruma_common::{
    OwnedMxcUri, OwnedRoomId, OwnedUserId,
    api::{auth_scheme::AccessToken, request, response},
    metadata,
    presence::PresenceState,
    serde::{Raw, duration::opt_ms},
};
use ruma_events::{AnyStrippedStateEvent, AnySyncStateEvent, AnySyncTimelineEvent, StateEventType};
use serde::{Deserialize, Serialize};

metadata! {
    method: POST,
    rate_limited: false,
    authentication: AccessToken,
    history: {
        stable("org.matrix.simplified_msc3575.stable") => "/_matrix/client/v4/sync",
    }
}

/// Request type for the `/sync` endpoint.
#[request]
#[derive(Default)]
pub struct Request {
    /// A unique string identifier for this connection to the server.
    ///
    /// Only one connection is allowed per given `conn_id`, empty or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conn_id: Option<String>,

    /// A point in time to continue a sync from.
    ///
    /// This is the `pos` from the previous response. `None` starts a new connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<String>,

    /// The maximum time to poll before responding to this request.
    ///
    /// `None` or zero means the server responds immediately. Ignored when [`Self::pos`] is
    /// `None`.
    #[serde(with = "opt_ms", default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Duration>,

    /// Controls whether the client is automatically marked as online by polling this API.
    ///
    /// Defaults to `PresenceState::Online`.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    pub set_presence: PresenceState,

    /// Lists of rooms we are interested in, keyed by an opaque identifier.
    ///
    /// The server rejects requests with more than 100 lists.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lists: BTreeMap<String, request::List>,

    /// Specific rooms we are interested in.
    ///
    /// It is useful to receive updates from rooms that are possibly out of range of all the
    /// lists (see [`Self::lists`]).
    ///
    /// The server rejects requests with more than 100 room subscriptions.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub room_subscriptions: BTreeMap<OwnedRoomId, request::RoomSubscription>,

    /// Extensions configuration.
    #[serde(default, skip_serializing_if = "request::Extensions::is_empty")]
    pub extensions: request::Extensions,
}

impl Request {
    /// Creates an empty `Request`.
    pub fn new() -> Self {
        Default::default()
    }
}

/// HTTP types related to a [`Request`].
pub mod request {
    use ruma_common::{OwnedRoomId, directory::RoomTypeFilter};
    use ruma_events::tag::TagName;

    use super::{BTreeMap, Deserialize, Serialize, StateEventType, UInt};

    /// A list of rooms (see [`super::Request::lists`]).
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct List {
        /// The range of rooms we're interested in, inclusive and 0-indexed.
        ///
        /// `None` means all the rooms that match the filters.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub range: Option<(UInt, UInt)>,

        /// Required state for each returned room.
        pub required_state: RequiredState,

        /// The maximum number of timeline events to return per room.
        pub timeline_limit: UInt,

        /// Filters to apply to the list.
        #[serde(default, skip_serializing_if = "ListFilters::is_empty")]
        pub filters: ListFilters,
    }

    impl List {
        /// Creates a new `List` with the given required state and timeline limit.
        pub fn new(required_state: RequiredState, timeline_limit: UInt) -> Self {
            Self { range: None, required_state, timeline_limit, filters: ListFilters::new() }
        }
    }

    /// The filters of a list (see [`List::filters`]).
    ///
    /// All fields are applied with _AND_ operators. The absence of fields
    /// implies no filter on that criteria: it does NOT imply `false`.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ListFilters {
        /// Whether to return DM rooms (as determined by the `m.direct` account data event).
        ///
        /// Defaults to [`FilterMode::Both`].
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub is_dm: FilterMode,

        /// Only list rooms that are direct children of any of these spaces, according to their
        /// `m.space.child` state events.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub spaces: Vec<OwnedRoomId>,

        /// Whether to return encrypted rooms (as determined by the existence of an
        /// `m.room.encryption` state event).
        ///
        /// Defaults to [`FilterMode::Both`].
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub is_encrypted: FilterMode,

        /// Whether to return rooms the user is invited to.
        ///
        /// Defaults to [`FilterMode::Both`].
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub is_invited: FilterMode,

        /// Only list rooms with these create-types, or all.
        ///
        /// If a room type is specified in both `room_types` and `not_room_types`,
        /// `not_room_types` wins and the corresponding rooms are not included.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub room_types: Vec<RoomTypeFilter>,

        /// Only list rooms that are not of these create-types, or all.
        ///
        /// If a room type is specified in both `room_types` and `not_room_types`,
        /// `not_room_types` wins and the corresponding rooms are not included.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub not_room_types: Vec<RoomTypeFilter>,

        /// Only list rooms with any of these tags, or all.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub tags: Vec<TagName>,

        /// Only list rooms with none of these tags, or all.
        ///
        /// Takes priority over `tags`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub not_tags: Vec<TagName>,
    }

    impl ListFilters {
        /// Creates an empty `ListFilters`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether no filter is set.
        pub fn is_empty(&self) -> bool {
            let Self {
                is_dm,
                spaces,
                is_encrypted,
                is_invited,
                room_types,
                not_room_types,
                tags,
                not_tags,
            } = self;

            *is_dm == FilterMode::Both
                && spaces.is_empty()
                && *is_encrypted == FilterMode::Both
                && *is_invited == FilterMode::Both
                && room_types.is_empty()
                && not_room_types.is_empty()
                && tags.is_empty()
                && not_tags.is_empty()
        }
    }

    /// How a filter of [`ListFilters`] applies to the rooms matching its criterion.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(from = "Option<bool>", into = "Option<bool>")]
    #[allow(clippy::exhaustive_enums)]
    pub enum FilterMode {
        /// Return both the rooms that match the criterion and the rooms that don't.
        #[default]
        Both,

        /// Return only the rooms that match the criterion.
        Only,

        /// Return only the rooms that don't match the criterion.
        Exclude,
    }

    impl From<Option<bool>> for FilterMode {
        fn from(value: Option<bool>) -> Self {
            match value {
                None => Self::Both,
                Some(true) => Self::Only,
                Some(false) => Self::Exclude,
            }
        }
    }

    impl From<FilterMode> for Option<bool> {
        fn from(value: FilterMode) -> Self {
            match value {
                FilterMode::Both => None,
                FilterMode::Only => Some(true),
                FilterMode::Exclude => Some(false),
            }
        }
    }

    /// A room subscription (see [`super::Request::room_subscriptions`]).
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RoomSubscription {
        /// Required state for the room.
        pub required_state: RequiredState,

        /// The maximum number of timeline events to return for the room.
        pub timeline_limit: UInt,
    }

    impl RoomSubscription {
        /// Creates a new `RoomSubscription` with the given required state and timeline limit.
        pub fn new(required_state: RequiredState, timeline_limit: UInt) -> Self {
            Self { required_state, timeline_limit }
        }
    }

    /// The set of state that the server should return for a room.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RequiredState {
        /// The state to return, unless filtered out by `exclude`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub include: Vec<RequiredStateElement>,

        /// The state to filter out before returning.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub exclude: Vec<RequiredStateElement>,

        /// Whether to enable lazy-loading of room members.
        ///
        /// Defaults to `false`.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub lazy_members: bool,
    }

    impl RequiredState {
        /// Creates an empty `RequiredState`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// A matcher for state events (see [`RequiredState`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RequiredStateElement {
        /// The event type to match.
        ///
        /// `None` matches all types.
        #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
        pub event_type: Option<StateEventType>,

        /// The state key to match.
        ///
        /// `None` matches all state keys.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub state_key: Option<String>,
    }

    impl RequiredStateElement {
        /// Creates a `RequiredStateElement` that matches all state events.
        pub fn new() -> Self {
            Default::default()
        }

        /// Only match state events with the given event type.
        pub fn with_event_type(mut self, event_type: StateEventType) -> Self {
            self.event_type = Some(event_type);
            self
        }

        /// Only match state events with the given state key.
        pub fn with_state_key(mut self, state_key: String) -> Self {
            self.state_key = Some(state_key);
            self
        }
    }

    /// Extensions configuration (see [`super::Request::extensions`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Extensions {
        /// Configure the to-device extension.
        #[serde(default, skip_serializing_if = "ToDevice::is_disabled")]
        pub to_device: ToDevice,

        /// Configure the E2EE extension.
        #[serde(default, skip_serializing_if = "E2EE::is_disabled")]
        pub e2ee: E2EE,

        /// Configure the account data extension.
        #[serde(default, skip_serializing_if = "AccountData::is_disabled")]
        pub account_data: AccountData,

        /// Configure the receipts extension.
        #[serde(default, skip_serializing_if = "Receipts::is_disabled")]
        pub receipts: Receipts,

        /// Configure the typing extension.
        #[serde(default, skip_serializing_if = "Typing::is_disabled")]
        pub typing: Typing,

        /// Configure the thread subscriptions extension.
        #[cfg(feature = "unstable-msc4308")]
        #[serde(
            default,
            skip_serializing_if = "ThreadSubscriptions::is_disabled",
            rename = "io.element.msc4308.thread_subscriptions"
        )]
        pub thread_subscriptions: ThreadSubscriptions,

        /// Configure the threads extension.
        #[cfg(feature = "unstable-msc4360")]
        #[serde(
            default,
            skip_serializing_if = "Threads::is_disabled",
            rename = "io.element.msc4360.threads"
        )]
        pub threads: Threads,

        /// Configure the profiles extension.
        #[cfg(feature = "unstable-msc4262")]
        #[serde(
            default,
            skip_serializing_if = "Profiles::is_disabled",
            rename = "org.matrix.msc4262.profiles"
        )]
        pub profiles: Profiles,

        /// Configure the sticky events extension.
        #[cfg(feature = "unstable-msc4480")]
        #[serde(
            default,
            skip_serializing_if = "StickyEvents::is_disabled",
            rename = "org.matrix.msc4354.sticky_events"
        )]
        pub sticky_events: StickyEvents,

        /// Configure the presence extension.
        #[cfg(feature = "unstable-msc4559")]
        #[serde(
            default,
            skip_serializing_if = "Presence::is_disabled",
            rename = "org.continuwuity.presence_v2.msc4559.presence"
        )]
        pub presence: Presence,

        /// Extensions may add further fields to the list.
        #[serde(flatten)]
        pub(crate) other: BTreeMap<String, serde_json::Value>,
    }

    impl Extensions {
        /// Creates an empty `Extensions`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether all the extensions are disabled.
        pub fn is_empty(&self) -> bool {
            let Self {
                to_device,
                e2ee,
                account_data,
                receipts,
                typing,
                #[cfg(feature = "unstable-msc4308")]
                thread_subscriptions,
                #[cfg(feature = "unstable-msc4360")]
                threads,
                #[cfg(feature = "unstable-msc4262")]
                profiles,
                #[cfg(feature = "unstable-msc4480")]
                sticky_events,
                #[cfg(feature = "unstable-msc4559")]
                presence,
                other,
            } = self;

            #[allow(unused_mut)]
            let mut empty = to_device.is_disabled()
                && e2ee.is_disabled()
                && account_data.is_disabled()
                && receipts.is_disabled()
                && typing.is_disabled()
                && other.is_empty();

            #[cfg(feature = "unstable-msc4308")]
            {
                empty &= thread_subscriptions.is_disabled();
            }

            #[cfg(feature = "unstable-msc4360")]
            {
                empty &= threads.is_disabled();
            }

            #[cfg(feature = "unstable-msc4262")]
            {
                empty &= profiles.is_disabled();
            }

            #[cfg(feature = "unstable-msc4480")]
            {
                empty &= sticky_events.is_disabled();
            }

            #[cfg(feature = "unstable-msc4559")]
            {
                empty &= presence.is_disabled();
            }

            empty
        }
    }

    /// To-device extension configuration.
    ///
    /// According to [MSC4538](https://github.com/matrix-org/matrix-spec-proposals/pull/4538).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ToDevice {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,

        /// Maximum number of to-device messages per response.
        ///
        /// Defaults to 100.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// Give messages since this token only.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub since: Option<String>,
    }

    impl ToDevice {
        /// Creates a disabled `ToDevice`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }

    /// E2EE extension configuration.
    ///
    /// According to [MSC4543](https://github.com/matrix-org/matrix-spec-proposals/pull/4543).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct E2EE {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,
    }

    impl E2EE {
        /// Creates a disabled `E2EE`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }

    /// Account data extension configuration.
    ///
    /// According to [MSC4548](https://github.com/matrix-org/matrix-spec-proposals/pull/4548).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct AccountData {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,

        /// The keys of the lists the extension applies to.
        ///
        /// `None` means all the lists, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// The room subscriptions the extension applies to.
        ///
        /// `None` means all the room subscriptions, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<OwnedRoomId>>,
    }

    impl AccountData {
        /// Creates a disabled `AccountData`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }

    /// Receipts extension configuration.
    ///
    /// According to [MSC4551](https://github.com/matrix-org/matrix-spec-proposals/pull/4551).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Receipts {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,

        /// The keys of the lists the extension applies to.
        ///
        /// `None` means all the lists, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// The room subscriptions the extension applies to.
        ///
        /// `None` means all the room subscriptions, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<OwnedRoomId>>,
    }

    impl Receipts {
        /// Creates a disabled `Receipts`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }

    /// Typing extension configuration.
    ///
    /// According to [MSC4508](https://github.com/matrix-org/matrix-spec-proposals/pull/4508).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Typing {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,

        /// The keys of the lists the extension applies to.
        ///
        /// `None` means all the lists, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// The room subscriptions the extension applies to.
        ///
        /// `None` means all the room subscriptions, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<OwnedRoomId>>,
    }

    impl Typing {
        /// Creates a disabled `Typing`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }

    /// Thread subscriptions extension configuration.
    ///
    /// According to [MSC4308](https://github.com/matrix-org/matrix-spec-proposals/pull/4308).
    #[cfg(feature = "unstable-msc4308")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThreadSubscriptions {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,

        /// Maximum number of thread subscription changes to receive in the response.
        ///
        /// Defaults to 100.
        /// Servers may impose a smaller limit than what is requested here.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,
    }

    #[cfg(feature = "unstable-msc4308")]
    impl ThreadSubscriptions {
        /// Creates a disabled `ThreadSubscriptions`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }

    /// Threads extension configuration.
    ///
    /// According to [MSC4360](https://github.com/matrix-org/matrix-spec-proposals/pull/4360).
    #[cfg(feature = "unstable-msc4360")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Threads {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,

        /// Whether to include the thread root events in the response.
        pub include_roots: bool,

        /// Maximum number of thread updates to receive in the response.
        ///
        /// Defaults to 100.
        /// Servers may impose a smaller limit than what is requested here.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,
    }

    #[cfg(feature = "unstable-msc4360")]
    impl Threads {
        /// Creates a disabled `Threads`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }

    /// User profiles extension configuration.
    ///
    /// According to [MSC4262](https://github.com/matrix-org/matrix-spec-proposals/pull/4262).
    #[cfg(feature = "unstable-msc4262")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Profiles {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,

        /// The profile fields to receive updates for.
        ///
        /// `None` means all the profile fields.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub fields: Option<Vec<ruma_common::profile::ProfileFieldName>>,

        /// The keys of the lists the extension applies to.
        ///
        /// `None` means all the lists, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// The room subscriptions the extension applies to.
        ///
        /// `None` means all the room subscriptions, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<OwnedRoomId>>,
    }

    #[cfg(feature = "unstable-msc4262")]
    impl Profiles {
        /// Creates a disabled `Profiles`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }

    /// Sticky events extension configuration.
    ///
    /// According to [MSC4480](https://github.com/matrix-org/matrix-spec-proposals/pull/4480).
    #[cfg(feature = "unstable-msc4480")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct StickyEvents {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,

        /// Maximum number of sticky events to return per response.
        ///
        /// Defaults to 100 (server-side) and the server may override it to a lower value.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// The keys of the lists the extension applies to.
        ///
        /// `None` means all the lists, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// The room subscriptions the extension applies to.
        ///
        /// `None` means all the room subscriptions, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<OwnedRoomId>>,
    }

    #[cfg(feature = "unstable-msc4480")]
    impl StickyEvents {
        /// Creates a disabled `StickyEvents`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }

    /// Presence extension configuration.
    ///
    /// According to [MSC4559](https://github.com/matrix-org/matrix-spec-proposals/pull/4559).
    #[cfg(feature = "unstable-msc4559")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Presence {
        /// Whether this extension is enabled.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub enabled: bool,

        /// The keys of the lists the extension applies to.
        ///
        /// `None` means all the lists, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// The room subscriptions the extension applies to.
        ///
        /// `None` means all the room subscriptions, and an empty list means none of them.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<OwnedRoomId>>,
    }

    #[cfg(feature = "unstable-msc4559")]
    impl Presence {
        /// Creates a disabled `Presence`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether this extension is disabled.
        pub fn is_disabled(&self) -> bool {
            !self.enabled
        }
    }
}

/// Response type for the `/sync` endpoint.
#[response]
pub struct Response {
    /// The token to supply in the `pos` field of the next `/sync` request
    /// (see [`Request::pos`]).
    pub pos: String,

    /// Resulting details of the lists.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lists: BTreeMap<String, response::List>,

    /// The updated rooms.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rooms: BTreeMap<OwnedRoomId, response::Room>,

    /// Extensions data.
    #[serde(default, skip_serializing_if = "response::Extensions::is_empty")]
    pub extensions: response::Extensions,
}

impl Response {
    /// Creates a new `Response` with the given `pos`.
    pub fn new(pos: String) -> Self {
        Self {
            pos,
            lists: Default::default(),
            rooms: Default::default(),
            extensions: Default::default(),
        }
    }
}

/// HTTP types related to a [`Response`].
pub mod response {
    #[cfg(any(feature = "unstable-msc4308", feature = "unstable-msc4360"))]
    use ruma_common::OwnedEventId;
    #[cfg(feature = "unstable-msc4559")]
    use ruma_common::presence::PresenceStatus;
    #[cfg(feature = "unstable-msc4262")]
    use ruma_common::profile::UserProfileUpdate;
    use ruma_common::{OneTimeKeyAlgorithm, serde::from_raw_json_value};
    use ruma_events::{
        AnyGlobalAccountDataEvent, AnyRoomAccountDataEvent, AnyToDeviceEvent,
        receipt::ReceiptEventContent, room::member::MembershipState, typing::TypingEventContent,
    };
    use serde::de::IgnoredAny;
    use serde_json::value::RawValue as RawJsonValue;

    #[cfg(feature = "unstable-msc4559")]
    use super::PresenceState;
    use super::{
        super::DeviceLists, AnyStrippedStateEvent, AnySyncStateEvent, AnySyncTimelineEvent,
        BTreeMap, Deserialize, JsOption, OwnedMxcUri, OwnedRoomId, OwnedUserId, Raw, Serialize,
        StateEventType, UInt,
    };
    #[cfg(feature = "unstable-msc4308")]
    use crate::threads::get_thread_subscriptions_changes::msc4308::{
        ThreadSubscription, ThreadUnsubscription,
    };

    /// The details of a list (see [`super::Response::lists`]).
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct List {
        /// The total number of rooms that match the filters of this list, ignoring the range.
        pub count: UInt,
    }

    impl List {
        /// Creates a new `List` with the given count.
        pub fn new(count: UInt) -> Self {
            Self { count }
        }
    }

    /// An updated room (see [`super::Response::rooms`]).
    #[derive(Clone, Debug, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Room {
        /// An integer that can be used to sort rooms based on their last activity.
        ///
        /// Greater means more recent. The value is opaque and may decrease.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bump_stamp: Option<UInt>,

        /// The current membership of the user in the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub membership: Option<MembershipState>,

        /// The keys of the lists that match this room.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub lists: Vec<String>,

        /// The data of the room, depending on the membership of the user.
        #[serde(flatten)]
        pub data: RoomData,
    }

    impl Room {
        /// Creates a new `Room` with the given data.
        pub fn new(data: RoomData) -> Self {
            Self { bump_stamp: None, membership: None, lists: Vec::new(), data }
        }
    }

    impl<'de> Deserialize<'de> for Room {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct RoomDeHelper {
                bump_stamp: Option<UInt>,
                membership: Option<MembershipState>,
                #[serde(default)]
                lists: Vec<String>,
            }

            let json = Box::<RawJsonValue>::deserialize(deserializer)?;
            let RoomDeHelper { bump_stamp, membership, lists } = from_raw_json_value(&json)?;
            let data = from_raw_json_value(&json)?;

            Ok(Self { bump_stamp, membership, lists, data })
        }
    }

    /// The data of a [`Room`].
    #[derive(Clone, Debug, Serialize)]
    #[serde(untagged)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum RoomData {
        /// The data of a room the user is or has been joined to.
        Full(FullRoomData),

        /// The data of a room the user has not been joined to, e.g. an invite or a knock.
        Stripped(StrippedRoomData),
    }

    impl<'de> Deserialize<'de> for RoomData {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct StrippedStateDeHelper {
                stripped_state: Option<IgnoredAny>,
            }

            let json = Box::<RawJsonValue>::deserialize(deserializer)?;
            let StrippedStateDeHelper { stripped_state } = from_raw_json_value(&json)?;

            if stripped_state.is_some() {
                Ok(Self::Stripped(from_raw_json_value(&json)?))
            } else {
                Ok(Self::Full(from_raw_json_value(&json)?))
            }
        }
    }

    impl From<FullRoomData> for RoomData {
        fn from(value: FullRoomData) -> Self {
            Self::Full(value)
        }
    }

    impl From<StrippedRoomData> for RoomData {
        fn from(value: StrippedRoomData) -> Self {
            Self::Stripped(value)
        }
    }

    /// The data of a room the user is or has been joined to (see [`RoomData::Full`]).
    ///
    /// When [`FullRoomData::initial`] is `false`, an omitted field means that it is unchanged.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct FullRoomData {
        /// The name as calculated by the server.
        ///
        /// `Null` if the room name was removed.
        #[serde(default, skip_serializing_if = "JsOption::is_undefined")]
        pub name: JsOption<String>,

        /// The avatar.
        ///
        /// `Null` if the room avatar was removed.
        #[serde(default, skip_serializing_if = "JsOption::is_undefined")]
        pub avatar: JsOption<OwnedMxcUri>,

        /// Heroes of the room, if the room has no name.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub heroes: Option<Vec<Hero>>,

        /// Whether it is a direct room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_dm: Option<bool>,

        /// Whether it is the first time this room is sent on this connection, or the client
        /// should replace all its room data with what is returned.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub initial: bool,

        /// Whether more historic events are returned because the timeline limit increased.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub expanded_timeline: bool,

        /// Changes in the current state of the room.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub required_state: Vec<RequiredStateEntry>,

        /// The latest events in the room.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub timeline: Vec<Raw<AnySyncTimelineEvent>>,

        /// The `prev_batch` allowing you to paginate through the messages
        /// before the given ones.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,

        /// Whether there are more events since the previous sync than were included in the
        /// timeline.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub limited: bool,

        /// The number of timeline events which have just occurred and are not
        /// historical.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub num_live: Option<UInt>,

        /// The number of users with membership of `join`, including the
        /// client’s own user ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub joined_count: Option<UInt>,

        /// The number of users with membership of `invite`.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub invited_count: Option<UInt>,
    }

    impl FullRoomData {
        /// Creates an empty `FullRoomData`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// The data of a room the user has not been joined to (see [`RoomData::Stripped`]).
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct StrippedRoomData {
        /// Stripped state events of the room.
        pub stripped_state: Vec<Raw<AnyStrippedStateEvent>>,
    }

    impl StrippedRoomData {
        /// Creates a new `StrippedRoomData` with the given stripped state.
        pub fn new(stripped_state: Vec<Raw<AnyStrippedStateEvent>>) -> Self {
            Self { stripped_state }
        }
    }

    /// A room hero (see [`FullRoomData::heroes`]).
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Hero {
        /// The user ID.
        pub user_id: OwnedUserId,

        /// The display name from the membership event, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub displayname: Option<String>,

        /// The avatar URL from the membership event, if set.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub avatar_url: Option<OwnedMxcUri>,
    }

    impl Hero {
        /// Creates a new `Hero` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, displayname: None, avatar_url: None }
        }
    }

    /// An entry of [`FullRoomData::required_state`].
    #[derive(Clone, Debug, Serialize)]
    #[serde(untagged)]
    #[allow(clippy::exhaustive_enums)]
    pub enum RequiredStateEntry {
        /// A state event.
        Event(Raw<AnySyncStateEvent>),

        /// A state entry that was deleted.
        Deleted(StateStub),
    }

    impl<'de> Deserialize<'de> for RequiredStateEntry {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct ContentDeHelper {
                #[serde(default)]
                content: JsOption<IgnoredAny>,
            }

            let json = Box::<RawJsonValue>::deserialize(deserializer)?;
            let ContentDeHelper { content } = from_raw_json_value(&json)?;

            if content.is_undefined() {
                Ok(Self::Deleted(from_raw_json_value(&json)?))
            } else {
                Ok(Self::Event(Raw::from_json(json)))
            }
        }
    }

    /// A state entry that was deleted (see [`RequiredStateEntry::Deleted`]).
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct StateStub {
        /// The type of the deleted state entry.
        #[serde(rename = "type")]
        pub event_type: StateEventType,

        /// The state key of the deleted state entry.
        pub state_key: String,
    }

    impl StateStub {
        /// Creates a new `StateStub` with the given event type and state key.
        pub fn new(event_type: StateEventType, state_key: String) -> Self {
            Self { event_type, state_key }
        }
    }

    /// Extensions data (see [`super::Response::extensions`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Extensions {
        /// To-device extension response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub to_device: Option<ToDevice>,

        /// E2EE extension response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub e2ee: Option<E2EE>,

        /// Account data extension response.
        #[serde(default, skip_serializing_if = "AccountData::is_empty")]
        pub account_data: AccountData,

        /// Receipts extension response.
        #[serde(default, skip_serializing_if = "Receipts::is_empty")]
        pub receipts: Receipts,

        /// Typing extension response.
        #[serde(default, skip_serializing_if = "Typing::is_empty")]
        pub typing: Typing,

        /// Thread subscriptions extension response.
        #[cfg(feature = "unstable-msc4308")]
        #[serde(
            default,
            skip_serializing_if = "ThreadSubscriptions::is_empty",
            rename = "io.element.msc4308.thread_subscriptions"
        )]
        pub thread_subscriptions: ThreadSubscriptions,

        /// Threads extension response.
        #[cfg(feature = "unstable-msc4360")]
        #[serde(
            default,
            skip_serializing_if = "Threads::is_empty",
            rename = "io.element.msc4360.threads"
        )]
        pub threads: Threads,

        /// Profiles extension response.
        #[cfg(feature = "unstable-msc4262")]
        #[serde(
            default,
            skip_serializing_if = "Profiles::is_empty",
            rename = "org.matrix.msc4262.profiles"
        )]
        pub profiles: Profiles,

        /// Sticky events extension response.
        #[cfg(feature = "unstable-msc4480")]
        #[serde(
            default,
            skip_serializing_if = "StickyEvents::is_empty",
            rename = "org.matrix.msc4354.sticky_events"
        )]
        pub sticky_events: StickyEvents,

        /// Presence extension response.
        #[cfg(feature = "unstable-msc4559")]
        #[serde(
            default,
            skip_serializing_if = "Presence::is_empty",
            rename = "org.continuwuity.presence_v2.msc4559.presence"
        )]
        pub presence: Presence,
    }

    impl Extensions {
        /// Creates an empty `Extensions`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether the extension data is empty.
        pub fn is_empty(&self) -> bool {
            let Self {
                to_device,
                e2ee,
                account_data,
                receipts,
                typing,
                #[cfg(feature = "unstable-msc4308")]
                thread_subscriptions,
                #[cfg(feature = "unstable-msc4360")]
                threads,
                #[cfg(feature = "unstable-msc4262")]
                profiles,
                #[cfg(feature = "unstable-msc4480")]
                sticky_events,
                #[cfg(feature = "unstable-msc4559")]
                presence,
            } = self;

            #[allow(unused_mut)]
            let mut empty = to_device.is_none()
                && e2ee.is_none()
                && account_data.is_empty()
                && receipts.is_empty()
                && typing.is_empty();

            #[cfg(feature = "unstable-msc4308")]
            {
                empty &= thread_subscriptions.is_empty();
            }

            #[cfg(feature = "unstable-msc4360")]
            {
                empty &= threads.is_empty();
            }

            #[cfg(feature = "unstable-msc4262")]
            {
                empty &= profiles.is_empty();
            }

            #[cfg(feature = "unstable-msc4480")]
            {
                empty &= sticky_events.is_empty();
            }

            #[cfg(feature = "unstable-msc4559")]
            {
                empty &= presence.is_empty();
            }

            empty
        }
    }

    /// To-device extension response.
    ///
    /// According to [MSC4538](https://github.com/matrix-org/matrix-spec-proposals/pull/4538).
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ToDevice {
        /// The token to send as `since` in the next request.
        pub next_batch: String,

        /// The messages for this device, in order of arrival.
        pub messages: Vec<Raw<AnyToDeviceEvent>>,
    }

    impl ToDevice {
        /// Creates a new `ToDevice` with the given `next_batch` and messages.
        pub fn new(next_batch: String, messages: Vec<Raw<AnyToDeviceEvent>>) -> Self {
            Self { next_batch, messages }
        }
    }

    /// E2EE extension response.
    ///
    /// According to [MSC4543](https://github.com/matrix-org/matrix-spec-proposals/pull/4543).
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct E2EE {
        /// Information on E2EE device updates.
        #[serde(default, skip_serializing_if = "DeviceLists::is_empty")]
        pub device_lists: DeviceLists,

        /// For each key algorithm, the number of unclaimed one-time keys currently held on the
        /// server for this device.
        ///
        /// An algorithm that is not listed has a count of 0.
        pub device_one_time_keys_count: BTreeMap<OneTimeKeyAlgorithm, UInt>,

        /// The key algorithms for which the device has an unused fallback key.
        pub device_unused_fallback_key_types: Vec<OneTimeKeyAlgorithm>,
    }

    impl E2EE {
        /// Creates a new `E2EE` with the given one-time keys count and unused fallback key types.
        pub fn new(
            device_one_time_keys_count: BTreeMap<OneTimeKeyAlgorithm, UInt>,
            device_unused_fallback_key_types: Vec<OneTimeKeyAlgorithm>,
        ) -> Self {
            Self {
                device_lists: DeviceLists::new(),
                device_one_time_keys_count,
                device_unused_fallback_key_types,
            }
        }
    }

    /// Account data extension response.
    ///
    /// According to [MSC4548](https://github.com/matrix-org/matrix-spec-proposals/pull/4548).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct AccountData {
        /// The global private data created by this user.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub global: Vec<Raw<AnyGlobalAccountDataEvent>>,

        /// The private data that this user has attached to each room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, Vec<Raw<AnyRoomAccountDataEvent>>>,
    }

    impl AccountData {
        /// Creates an empty `AccountData`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether all fields are empty.
        pub fn is_empty(&self) -> bool {
            let Self { global, rooms } = self;
            global.is_empty() && rooms.is_empty()
        }
    }

    /// Receipts extension response.
    ///
    /// According to [MSC4551](https://github.com/matrix-org/matrix-spec-proposals/pull/4551).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Receipts {
        /// The receipts in each room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, Raw<ReceiptEventContent>>,
    }

    impl Receipts {
        /// Creates an empty `Receipts`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether all fields are empty.
        pub fn is_empty(&self) -> bool {
            let Self { rooms } = self;
            rooms.is_empty()
        }
    }

    /// Typing extension response.
    ///
    /// According to [MSC4508](https://github.com/matrix-org/matrix-spec-proposals/pull/4508).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Typing {
        /// The users currently typing in each room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, TypingEventContent>,
    }

    impl Typing {
        /// Creates an empty `Typing`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether all fields are empty.
        pub fn is_empty(&self) -> bool {
            let Self { rooms } = self;
            rooms.is_empty()
        }
    }

    /// Thread subscriptions extension response.
    ///
    /// According to [MSC4308](https://github.com/matrix-org/matrix-spec-proposals/pull/4308).
    #[cfg(feature = "unstable-msc4308")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThreadSubscriptions {
        /// New thread subscriptions.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub subscribed: BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, ThreadSubscription>>,

        /// New thread unsubscriptions.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub unsubscribed: BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, ThreadUnsubscription>>,

        /// A token that can be used to backpaginate (via the companion endpoint) other thread
        /// subscription changes that occurred since the last sync, but that were not included in
        /// this response.
        ///
        /// Only set when there are more changes to fetch.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,
    }

    #[cfg(feature = "unstable-msc4308")]
    impl ThreadSubscriptions {
        /// Creates an empty `ThreadSubscriptions`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            let Self { subscribed, unsubscribed, prev_batch } = self;
            subscribed.is_empty() && unsubscribed.is_empty() && prev_batch.is_none()
        }
    }

    /// Threads extension response.
    ///
    /// According to [MSC4360](https://github.com/matrix-org/matrix-spec-proposals/pull/4360).
    #[cfg(feature = "unstable-msc4360")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Threads {
        /// The updated threads, by room ID and thread root event ID.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub updates: BTreeMap<OwnedRoomId, BTreeMap<OwnedEventId, ThreadUpdate>>,

        /// A token to paginate the thread updates that were not included in this response, with
        /// the `/thread_updates` endpoint.
        ///
        /// Only set when there are more updates to fetch.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,
    }

    #[cfg(feature = "unstable-msc4360")]
    impl Threads {
        /// Creates an empty `Threads`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            let Self { updates, prev_batch } = self;
            updates.is_empty() && prev_batch.is_none()
        }
    }

    /// An update to a thread in the threads extension response.
    ///
    /// According to [MSC4360](https://github.com/matrix-org/matrix-spec-proposals/pull/4360).
    #[cfg(feature = "unstable-msc4360")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThreadUpdate {
        /// The thread root event, with the bundled `m.thread` aggregation.
        ///
        /// Only present if `include_roots` is set in the request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub thread_root: Option<Raw<AnySyncTimelineEvent>>,

        /// A token to paginate the other updates in this thread with the `/relations` endpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,
    }

    #[cfg(feature = "unstable-msc4360")]
    impl ThreadUpdate {
        /// Creates an empty `ThreadUpdate`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// Profiles extension response.
    ///
    /// According to [MSC4262](https://github.com/matrix-org/matrix-spec-proposals/pull/4262).
    #[cfg(feature = "unstable-msc4262")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Profiles {
        /// Profile updates keyed by user ID.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub users: BTreeMap<OwnedUserId, UserProfileUpdate>,
    }

    #[cfg(feature = "unstable-msc4262")]
    impl Profiles {
        /// Creates an empty `Profiles`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether all fields are empty.
        pub fn is_empty(&self) -> bool {
            let Self { users } = self;
            users.is_empty()
        }
    }

    /// Sticky events extension response.
    ///
    /// According to [MSC4480](https://github.com/matrix-org/matrix-spec-proposals/pull/4480).
    #[cfg(feature = "unstable-msc4480")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct StickyEvents {
        /// The sticky events, grouped by room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, StickyEventsRoom>,
    }

    #[cfg(feature = "unstable-msc4480")]
    impl StickyEvents {
        /// Creates an empty `StickyEvents`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether all fields are empty.
        pub fn is_empty(&self) -> bool {
            let Self { rooms } = self;
            rooms.is_empty()
        }
    }

    /// Sticky events for a single room in the sticky events extension response.
    #[cfg(feature = "unstable-msc4480")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct StickyEventsRoom {
        /// The sticky events for this room.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub events: Vec<Raw<AnySyncTimelineEvent>>,
    }

    #[cfg(feature = "unstable-msc4480")]
    impl StickyEventsRoom {
        /// Creates an empty `StickyEventsRoom`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// An update to a user's presence.
    ///
    /// According to [MSC4559](https://github.com/matrix-org/matrix-spec-proposals/pull/4559).
    #[cfg(feature = "unstable-msc4559")]
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct PresenceUpdate {
        /// The user's presence state.
        ///
        /// Defaults to `PresenceState::Offline`.
        #[serde(default = "presence_offline", skip_serializing_if = "is_presence_offline")]
        pub presence: PresenceState,

        /// The status information for this user's presence, using the format from [MSC4532].
        ///
        /// [MSC4532]: https://github.com/matrix-org/matrix-spec-proposals/pull/4532
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub status: PresenceStatus,
    }

    #[cfg(feature = "unstable-msc4559")]
    impl PresenceUpdate {
        /// Creates a new `PresenceUpdate` with the `offline` presence state and no status.
        pub fn new() -> Self {
            Self { presence: PresenceState::Offline, status: PresenceStatus::default() }
        }
    }

    #[cfg(feature = "unstable-msc4559")]
    impl Default for PresenceUpdate {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(feature = "unstable-msc4559")]
    fn presence_offline() -> PresenceState {
        PresenceState::Offline
    }

    #[cfg(feature = "unstable-msc4559")]
    fn is_presence_offline(presence: &PresenceState) -> bool {
        *presence == PresenceState::Offline
    }

    /// Presence extension response.
    ///
    /// According to [MSC4559](https://github.com/matrix-org/matrix-spec-proposals/pull/4559).
    #[cfg(feature = "unstable-msc4559")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Presence {
        /// Presence updates keyed by user ID.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub user_ids: BTreeMap<OwnedUserId, PresenceUpdate>,
    }

    #[cfg(feature = "unstable-msc4559")]
    impl Presence {
        /// Creates an empty `Presence`.
        pub fn new() -> Self {
            Default::default()
        }

        /// Whether all fields are empty.
        pub fn is_empty(&self) -> bool {
            let Self { user_ids } = self;
            user_ids.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use js_int::uint;
    use ruma_common::{assert_to_canonical_json_eq, owned_room_id};
    use ruma_events::StateEventType;
    use serde_json::{from_value as from_json_value, json};
    use strass::assert_let;

    use super::{
        request::{
            Extensions, FilterMode, List, ListFilters, RequiredState, RequiredStateElement,
            RoomSubscription,
        },
        response::{self, RequiredStateEntry, Room, RoomData, StateStub},
    };

    #[test]
    fn serialize_list() {
        let mut required_state = RequiredState::new();
        required_state.include.push(
            RequiredStateElement::new()
                .with_event_type(StateEventType::RoomCreate)
                .with_state_key(String::new()),
        );
        required_state.lazy_members = true;

        let mut list = List::new(required_state, uint!(10));
        list.range = Some((uint!(0), uint!(99)));

        assert_to_canonical_json_eq!(
            list,
            json!({
                "range": [0, 99],
                "required_state": {
                    "include": [{ "type": "m.room.create", "state_key": "" }],
                    "lazy_members": true,
                },
                "timeline_limit": 10,
            })
        );
    }

    #[test]
    fn serialize_room_subscription_all_state() {
        let mut required_state = RequiredState::new();
        required_state.include.push(RequiredStateElement::new());
        let subscription = RoomSubscription::new(required_state, uint!(10));

        assert_to_canonical_json_eq!(
            subscription,
            json!({
                "required_state": { "include": [{}] },
                "timeline_limit": 10,
            })
        );
    }

    #[test]
    fn list_filters_serde() {
        let mut filters = ListFilters::new();
        filters.is_dm = FilterMode::Only;
        filters.is_invited = FilterMode::Exclude;

        assert_to_canonical_json_eq!(filters, json!({ "is_dm": true, "is_invited": false }));

        let filters: ListFilters =
            from_json_value(json!({ "is_dm": true, "is_invited": false })).unwrap();
        assert_eq!(filters.is_dm, FilterMode::Only);
        assert_eq!(filters.is_encrypted, FilterMode::Both);
        assert_eq!(filters.is_invited, FilterMode::Exclude);
    }

    #[test]
    fn deserialize_full_room() {
        let room: Room = from_json_value(json!({
            "name": null,
            "initial": true,
            "membership": "join",
            "lists": ["foo-list"],
            "required_state": [
                {
                    "content": {},
                    "event_id": "$create:example.com",
                    "origin_server_ts": 1,
                    "sender": "@alice:example.com",
                    "state_key": "",
                    "type": "m.room.create",
                },
                { "type": "m.room.topic", "state_key": "" },
            ],
        }))
        .unwrap();

        assert_eq!(room.lists, ["foo-list"]);
        assert_let!(RoomData::Full(data) = room.data);
        assert!(data.name.is_null());
        assert!(data.initial);
        assert!(!data.expanded_timeline);
        assert_eq!(data.required_state.len(), 2);
        assert_let!(RequiredStateEntry::Event(_) = &data.required_state[0]);
        assert_let!(RequiredStateEntry::Deleted(stub) = &data.required_state[1]);
        assert_eq!(stub.event_type, StateEventType::RoomTopic);
        assert_eq!(stub.state_key, "");
    }

    #[test]
    fn deserialize_stripped_room() {
        let room: Room = from_json_value(json!({
            "bump_stamp": 4,
            "membership": "invite",
            "stripped_state": [],
        }))
        .unwrap();

        assert_eq!(room.bump_stamp, Some(uint!(4)));
        assert_let!(RoomData::Stripped(data) = room.data);
        assert_eq!(data.stripped_state.len(), 0);
    }

    #[test]
    fn serialize_extensions() {
        let mut extensions = Extensions::new();
        extensions.typing.enabled = true;
        extensions.typing.lists = Some(vec![]);
        extensions.typing.rooms = Some(vec![owned_room_id!("!a:example.com")]);

        assert_to_canonical_json_eq!(
            extensions,
            json!({ "typing": { "enabled": true, "lists": [], "rooms": ["!a:example.com"] } })
        );
    }

    #[test]
    fn deserialize_extensions_response() {
        let extensions: response::Extensions = from_json_value(json!({
            "to_device": { "next_batch": "1", "messages": [] },
            "e2ee": {
                "device_one_time_keys_count": { "signed_curve25519": 50 },
                "device_unused_fallback_key_types": ["signed_curve25519"],
            },
            "receipts": {
                "rooms": {
                    "!a:example.com": {
                        "$e:example.com": { "m.read": { "@b:example.com": { "ts": 1 } } },
                    },
                },
            },
            "typing": { "rooms": { "!a:example.com": { "user_ids": ["@b:example.com"] } } },
        }))
        .unwrap();

        assert_eq!(extensions.to_device.unwrap().next_batch, "1");
        assert_eq!(extensions.e2ee.unwrap().device_unused_fallback_key_types.len(), 1);
        assert_eq!(extensions.receipts.rooms.len(), 1);
        let typing = &extensions.typing.rooms["!a:example.com"];
        assert_eq!(typing.user_ids, ["@b:example.com"]);

        // `device_one_time_keys_count` and `device_unused_fallback_key_types` are required.
        from_json_value::<response::Extensions>(json!({ "e2ee": {} })).unwrap_err();
    }

    #[test]
    fn deserialize_null_content_as_event() {
        let entry: RequiredStateEntry = from_json_value(json!({
            "content": null,
            "type": "m.room.topic",
            "state_key": "",
        }))
        .unwrap();

        assert_let!(RequiredStateEntry::Event(_) = entry);
    }

    #[test]
    fn serialize_deleted_state() {
        let entry =
            RequiredStateEntry::Deleted(StateStub::new(StateEventType::RoomTopic, String::new()));

        assert_to_canonical_json_eq!(entry, json!({ "type": "m.room.topic", "state_key": "" }));
    }

    #[cfg(feature = "unstable-msc4480")]
    #[test]
    fn deserialize_sticky_events() {
        let sticky_events: response::StickyEvents = from_json_value(json!({
            "rooms": {
                "!room:example.com": {
                    "events": [{
                        "content": { "body": "sticky", "msgtype": "m.text" },
                        "event_id": "$1:example.com",
                        "origin_server_ts": 1,
                        "sender": "@alice:example.com",
                        "type": "m.room.message",
                        "sticky": { "duration_ms": 300_000 },
                    }]
                }
            }
        }))
        .unwrap();

        assert_eq!(sticky_events.rooms.len(), 1);
        let room = sticky_events.rooms.values().next().unwrap();
        assert_eq!(room.events.len(), 1);
    }

    #[cfg(feature = "unstable-msc4360")]
    #[test]
    fn threads_extension_serde() {
        use super::request;

        let mut extensions = Extensions::new();
        extensions.threads = request::Threads { enabled: true, include_roots: true, limit: None };
        assert_to_canonical_json_eq!(
            extensions,
            json!({ "io.element.msc4360.threads": { "enabled": true, "include_roots": true } })
        );

        let extensions: response::Extensions = from_json_value(json!({
            "io.element.msc4360.threads": {
                "updates": {
                    "!room:example.com": {
                        "$root:example.com": { "prev_batch": "t1" },
                    },
                },
                "prev_batch": "t2",
            },
        }))
        .unwrap();

        let threads = extensions.threads;
        let update = &threads.updates["!room:example.com"]["$root:example.com"];
        assert!(update.thread_root.is_none());
        assert_eq!(update.prev_batch.as_deref(), Some("t1"));
        assert_eq!(threads.prev_batch.as_deref(), Some("t2"));
    }

    #[cfg(feature = "unstable-msc4559")]
    #[test]
    fn presence_update_defaults() {
        use ruma_common::presence::PresenceState;

        let update: response::PresenceUpdate = from_json_value(json!({})).unwrap();
        assert_eq!(update.presence, PresenceState::Offline);
        assert_to_canonical_json_eq!(update, json!({}));
    }
}

#[cfg(all(test, feature = "client"))]
mod client_tests {
    use std::{borrow::Cow, time::Duration};

    use ruma_common::{
        api::{
            FeatureFlag, MatrixVersion, OutgoingRequestExt as _, SupportedVersions,
            auth_scheme::SendAccessToken,
        },
        assert_to_canonical_json_eq,
    };
    use serde_json::{Value as JsonValue, from_slice as from_json_slice, json};

    use super::Request;

    #[test]
    fn serialize_request() {
        let supported = SupportedVersions {
            versions: [MatrixVersion::V1_19].into(),
            features: [FeatureFlag::Msc4186Stable].into(),
        };
        let mut request = Request::new();
        request.conn_id = Some("main".to_owned());
        request.pos = Some("s58_224".to_owned());
        request.timeout = Some(Duration::from_millis(30000));

        let req: http::Request<Vec<u8>> = request
            .try_into_http_request(
                "https://homeserver.tld",
                SendAccessToken::IfRequired("auth_tok"),
                Cow::Owned(supported),
            )
            .unwrap();

        assert_eq!(req.uri().path(), "/_matrix/client/v4/sync");
        assert_eq!(req.uri().query(), None);
        assert_to_canonical_json_eq!(
            from_json_slice::<JsonValue>(req.body()).unwrap(),
            json!({ "conn_id": "main", "pos": "s58_224", "timeout": 30000 })
        );
    }
}
