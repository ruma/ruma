//! `/v4/` ([MSC])
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
    /// Only one sliding sync connection is allowed per given `conn_id`, empty or not.
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lists: BTreeMap<String, request::List>,

    /// Specific rooms we are interested in.
    ///
    /// It is useful to receive updates from rooms that are possibly out of range of all the
    /// lists (see [`Self::lists`]).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub room_subscriptions: BTreeMap<OwnedRoomId, request::RoomSubscription>,

    /// Extensions.
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

    #[cfg(feature = "unstable-msc4308")]
    pub use super::super::extensions::request::ThreadSubscriptions;
    #[cfg(feature = "unstable-msc4360")]
    pub use super::super::extensions::request::Threads;
    pub use super::super::extensions::request::{E2EE, ToDevice};
    use super::{BTreeMap, Deserialize, Serialize, StateEventType, UInt};

    /// A sliding sync list request (see [`super::Request::lists`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct List {
        /// The range of rooms we're interested in, inclusive and 0-indexed.
        ///
        /// `None` means all the rooms that match the filters.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub range: Option<(UInt, UInt)>,

        /// The details to be included per room.
        #[serde(flatten)]
        pub room_details: RoomDetails,

        /// Filters to apply to the list.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filters: Option<ListFilters>,
    }

    /// A sliding sync list request filters (see [`List::filters`]).
    ///
    /// All fields are applied with _AND_ operators. The absence of fields
    /// implies no filter on that criteria: it does NOT imply `false`.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ListFilters {
        /// Whether to return only DM rooms (as determined by the `m.direct` account data event),
        /// only non-DM rooms, or both.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_dm: Option<bool>,

        /// Only list rooms that are direct children of any of these spaces, according to their
        /// `m.space.child` state events.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub spaces: Vec<OwnedRoomId>,

        /// Whether to return only encrypted rooms (as determined by the existence of an
        /// `m.room.encryption` state event), only unencrypted rooms, or both.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_encrypted: Option<bool>,

        /// Whether to return only rooms the user is invited to, no rooms the user is invited to,
        /// or both.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_invited: Option<bool>,

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

    /// Sliding sync request room subscription (see [`super::Request::room_subscriptions`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RoomSubscription {
        /// Required state for each returned room.
        pub required_state: RequiredState,

        /// The maximum number of timeline events to return per room.
        pub timeline_limit: UInt,
    }

    /// Sliding sync request room details (see [`List::room_details`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RoomDetails {
        /// Required state for each returned room.
        pub required_state: RequiredState,

        /// The maximum number of timeline events to return per room.
        pub timeline_limit: UInt,
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
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub lazy_members: bool,
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
        /// Creates a `RequiredStateElement` that matches the given event type and state key.
        pub fn new(event_type: StateEventType, state_key: String) -> Self {
            Self { event_type: Some(event_type), state_key: Some(state_key) }
        }
    }

    /// Sliding sync request extensions (see [`super::Request::extensions`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Extensions {
        /// Configure the to-device extension.
        #[serde(default, skip_serializing_if = "ToDevice::is_empty")]
        pub to_device: ToDevice,

        /// Configure the E2EE extension.
        #[serde(default, skip_serializing_if = "E2EE::is_empty")]
        pub e2ee: E2EE,

        /// Configure the account data extension.
        #[serde(default, skip_serializing_if = "AccountData::is_empty")]
        pub account_data: AccountData,

        /// Configure the receipts extension.
        #[serde(default, skip_serializing_if = "Receipts::is_empty")]
        pub receipts: Receipts,

        /// Configure the typing extension.
        #[serde(default, skip_serializing_if = "Typing::is_empty")]
        pub typing: Typing,

        /// Configure the thread subscriptions extension.
        #[cfg(feature = "unstable-msc4308")]
        #[serde(
            default,
            skip_serializing_if = "ThreadSubscriptions::is_empty",
            rename = "io.element.msc4308.thread_subscriptions"
        )]
        pub thread_subscriptions: ThreadSubscriptions,

        /// Configure the threads extension.
        #[cfg(feature = "unstable-msc4360")]
        #[serde(
            default,
            skip_serializing_if = "Threads::is_empty",
            rename = "io.element.msc4360.threads"
        )]
        pub threads: Threads,

        /// Configure the profiles extension.
        #[cfg(feature = "unstable-msc4262")]
        #[serde(
            default,
            skip_serializing_if = "Profiles::is_empty",
            rename = "org.matrix.msc4262.profiles"
        )]
        pub profiles: Profiles,

        /// Configure the sticky events extension.
        #[cfg(feature = "unstable-msc4480")]
        #[serde(
            default,
            skip_serializing_if = "StickyEvents::is_empty",
            rename = "org.matrix.msc4354.sticky_events"
        )]
        pub sticky_events: StickyEvents,

        /// Configure the presence extension.
        #[cfg(feature = "unstable-msc4559")]
        #[serde(
            default,
            skip_serializing_if = "Presence::is_empty",
            rename = "org.continuwuity.presence_v2.msc4559.presence"
        )]
        pub presence: Presence,

        /// Extensions may add further fields to the list.
        #[serde(flatten)]
        pub(crate) other: BTreeMap<String, serde_json::Value>,
    }

    impl Extensions {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            #[allow(unused_mut)]
            let mut empty = self.to_device.is_empty()
                && self.e2ee.is_empty()
                && self.account_data.is_empty()
                && self.receipts.is_empty()
                && self.typing.is_empty()
                && self.other.is_empty();

            #[cfg(feature = "unstable-msc4308")]
            {
                empty = empty && self.thread_subscriptions.is_empty();
            }

            #[cfg(feature = "unstable-msc4360")]
            {
                empty = empty && self.threads.is_empty();
            }

            #[cfg(feature = "unstable-msc4262")]
            {
                empty = empty && self.profiles.is_empty();
            }

            #[cfg(feature = "unstable-msc4480")]
            {
                empty = empty && self.sticky_events.is_empty();
            }

            #[cfg(feature = "unstable-msc4559")]
            {
                empty = empty && self.presence.is_empty();
            }

            empty
        }
    }

    /// Account data extension.
    ///
    /// According to [MSC4548](https://github.com/matrix-org/matrix-spec-proposals/pull/4548).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct AccountData {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Receipts extension.
    ///
    /// According to [MSC4551](https://github.com/matrix-org/matrix-spec-proposals/pull/4551).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Receipts {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Typing extension.
    ///
    /// According to [MSC4508](https://github.com/matrix-org/matrix-spec-proposals/pull/4508).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Typing {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// User profiles extension.
    ///
    /// According to [MSC4262](https://github.com/matrix-org/matrix-spec-proposals/pull/4262).
    #[cfg(feature = "unstable-msc4262")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Profiles {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Sticky events extension.
    ///
    /// According to [MSC4480](https://github.com/matrix-org/matrix-spec-proposals/pull/4480).
    #[cfg(feature = "unstable-msc4480")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct StickyEvents {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none() && self.limit.is_none()
        }
    }

    /// Presence extension.
    ///
    /// According to [MSC4559](https://github.com/matrix-org/matrix-spec-proposals/pull/4559).
    #[cfg(feature = "unstable-msc4559")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Presence {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
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

    /// Extensions.
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
    use ruma_common::{OneTimeKeyAlgorithm, serde::from_raw_json_value};
    use ruma_events::{
        AnyToDeviceEvent, receipt::ReceiptEventContent, room::member::MembershipState,
        typing::TypingEventContent,
    };
    use serde::de::IgnoredAny;
    use serde_json::value::RawValue as RawJsonValue;

    pub use super::super::extensions::response::AccountData;
    #[cfg(feature = "unstable-msc4262")]
    pub use super::super::extensions::response::Profiles;
    #[cfg(feature = "unstable-msc4308")]
    pub use super::super::extensions::response::ThreadSubscriptions;
    #[cfg(feature = "unstable-msc4360")]
    pub use super::super::extensions::response::Threads;
    #[cfg(feature = "unstable-msc4559")]
    pub use super::super::extensions::response::{Presence, PresenceUpdate};
    #[cfg(feature = "unstable-msc4480")]
    pub use super::super::extensions::response::{StickyEvents, StickyEventsRoom};
    use super::{
        super::DeviceLists, AnyStrippedStateEvent, AnySyncStateEvent, AnySyncTimelineEvent,
        BTreeMap, Deserialize, JsOption, OwnedMxcUri, OwnedRoomId, OwnedUserId, Raw, Serialize,
        StateEventType, UInt,
    };

    /// A sliding sync response list (see [`super::Response::lists`]).
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct List {
        /// The total number of rooms that match the filters of this list, ignoring the range.
        pub count: UInt,
    }

    /// A sliding sync response updated room (see [`super::Response::rooms`]).
    ///
    /// When [`Room::initial`] is `false`, an omitted field means that it is unchanged.
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
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

        /// Stripped state events, for rooms the user is not joined to (e.g. invites or knocks).
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stripped_state: Option<Vec<Raw<AnyStrippedStateEvent>>>,
    }

    impl Room {
        /// Creates an empty `Room`.
        pub fn new() -> Self {
            Default::default()
        }
    }

    /// A sliding sync response room hero (see [`Room::heroes`]).
    #[derive(Clone, Debug, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Hero {
        /// The user ID.
        pub user_id: OwnedUserId,

        /// The display name from the membership event, if set.
        #[serde(rename = "displayname", skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        /// The avatar URL from the membership event, if set.
        #[serde(rename = "avatar_url", skip_serializing_if = "Option::is_none")]
        pub avatar: Option<OwnedMxcUri>,
    }

    impl Hero {
        /// Creates a new `Hero` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, name: None, avatar: None }
        }
    }

    /// An entry of [`Room::required_state`].
    #[derive(Clone, Debug)]
    #[allow(clippy::exhaustive_enums)]
    pub enum RequiredStateEntry {
        /// A state event.
        Event(Raw<AnySyncStateEvent>),

        /// A state entry that was deleted.
        Deleted(StateStub),
    }

    impl Serialize for RequiredStateEntry {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::Event(event) => event.serialize(serializer),
                Self::Deleted(stub) => stub.serialize(serializer),
            }
        }
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

    /// Extensions responses.
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
        /// Whether the extension data is empty.
        pub fn is_empty(&self) -> bool {
            #[allow(unused_mut)]
            let mut empty = self.to_device.is_none()
                && self.e2ee.is_none()
                && self.account_data.is_empty()
                && self.receipts.is_empty()
                && self.typing.is_empty();

            #[cfg(feature = "unstable-msc4308")]
            {
                empty = empty && self.thread_subscriptions.is_empty();
            }

            #[cfg(feature = "unstable-msc4360")]
            {
                empty = empty && self.threads.is_empty();
            }

            #[cfg(feature = "unstable-msc4262")]
            {
                empty = empty && self.profiles.is_empty();
            }

            #[cfg(feature = "unstable-msc4480")]
            {
                empty = empty && self.sticky_events.is_empty();
            }

            #[cfg(feature = "unstable-msc4559")]
            {
                empty = empty && self.presence.is_empty();
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
        /// Creates a new `ToDevice` with the given `next_batch` and no messages.
        pub fn new(next_batch: String) -> Self {
            Self { next_batch, messages: Vec::new() }
        }
    }

    /// E2EE extension response.
    ///
    /// According to [MSC4543](https://github.com/matrix-org/matrix-spec-proposals/pull/4543).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
        /// Creates an empty `E2EE`.
        pub fn new() -> Self {
            Default::default()
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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.rooms.is_empty()
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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.rooms.is_empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use js_int::uint;
    use ruma_common::{assert_to_canonical_json_eq, owned_room_id, owned_user_id};
    use ruma_events::StateEventType;
    use serde_json::{from_value as from_json_value, json};

    use super::{
        request::{
            Extensions, List, RequiredState, RequiredStateElement, RoomDetails, RoomSubscription,
        },
        response::{self, RequiredStateEntry, Room, StateStub},
    };

    #[test]
    fn serialize_list() {
        let list = List {
            range: Some((uint!(0), uint!(99))),
            room_details: RoomDetails {
                required_state: RequiredState {
                    include: vec![RequiredStateElement::new(
                        StateEventType::RoomCreate,
                        String::new(),
                    )],
                    exclude: vec![],
                    lazy_members: true,
                },
                timeline_limit: uint!(10),
            },
            filters: None,
        };

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
        let subscription = RoomSubscription {
            required_state: RequiredState {
                include: vec![RequiredStateElement::default()],
                ..Default::default()
            },
            timeline_limit: uint!(10),
        };

        assert_to_canonical_json_eq!(
            subscription,
            json!({
                "required_state": { "include": [{}] },
                "timeline_limit": 10,
            })
        );
    }

    #[test]
    fn deserialize_room() {
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
            "stripped_state": [],
        }))
        .unwrap();

        assert!(room.name.is_null());
        assert!(room.initial);
        assert!(!room.expanded_timeline);
        assert_eq!(room.lists, ["foo-list"]);
        assert_eq!(room.required_state.len(), 2);
        assert_matches!(&room.required_state[0], RequiredStateEntry::Event(_));
        assert_matches!(&room.required_state[1], RequiredStateEntry::Deleted(stub) => {
            assert_eq!(stub.event_type, StateEventType::RoomTopic);
            assert_eq!(stub.state_key, "");
        });
        assert_eq!(room.stripped_state.unwrap().len(), 0);
    }

    #[test]
    fn serialize_extensions() {
        let mut extensions = Extensions::default();
        extensions.typing.enabled = Some(true);
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
        let typing = &extensions.typing.rooms[&owned_room_id!("!a:example.com")];
        assert_eq!(typing.user_ids, [owned_user_id!("@b:example.com")]);

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

        assert_matches!(entry, RequiredStateEntry::Event(_));
    }

    #[test]
    fn serialize_deleted_state() {
        let entry =
            RequiredStateEntry::Deleted(StateStub::new(StateEventType::RoomTopic, String::new()));

        assert_to_canonical_json_eq!(entry, json!({ "type": "m.room.topic", "state_key": "" }));
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
