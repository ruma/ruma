//! `POST /_matrix/client/unstable/org.matrix.simplified_msc3575/sync` ([MSC4186])
//!
//! A simplified version of sliding sync ([MSC3575]).
//!
//! Get all new events in a sliding window of rooms since the last sync or a given point in time.
//!
//! [MSC3575]: https://github.com/matrix-org/matrix-spec-proposals/pull/3575
//! [MSC4186]: https://github.com/matrix-org/matrix-spec-proposals/pull/4186

use std::{collections::BTreeMap, time::Duration};

use js_int::UInt;
use js_option::JsOption;
use ruma_common::{
    api::{request, response, Metadata},
    metadata,
    presence::PresenceState,
    serde::{duration::opt_ms, Raw},
    OwnedMxcUri, OwnedRoomId, OwnedUserId,
};
use ruma_events::{AnySyncStateEvent, AnySyncTimelineEvent, StateEventType};
use serde::{Deserialize, Serialize};

use super::UnreadNotificationsCount;

const METADATA: Metadata = metadata! {
    method: POST,
    rate_limited: false,
    authentication: AccessToken,
    history: {
        unstable("org.matrix.simplified_msc3575") => "/_matrix/client/unstable/org.matrix.simplified_msc3575/sync",
        // 1.4 => "/_matrix/client/v5/sync",
    }
};

/// Request type for the `/sync` endpoint.
#[request(error = crate::Error)]
#[derive(Default)]
pub struct Request {
    /// A point in time to continue a sync from.
    ///
    /// This is an opaque value taken from the `pos` field of a previous `/sync`
    /// response. A `None` value asks the server to start a new _session_ (mind
    /// it can be costly)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ruma_api(query)]
    pub pos: Option<String>,

    /// A unique string identifier for this connection to the server.
    ///
    /// If this is missing, only one sliding sync connection can be made to
    /// the server at any one time. Clients need to set this to allow more
    /// than one connection concurrently, so the server can distinguish between
    /// connections. This must be provided with every request, if your client
    /// needs more than one concurrent connection.
    ///
    /// Limitation: it must not contain more than 16 chars, due to it being
    /// required with every request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conn_id: Option<String>,

    /// Allows clients to know what request params reached the server,
    /// functionally similar to txn IDs on `/send` for events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_id: Option<String>,

    /// The maximum time to poll before responding to this request.
    ///
    /// `None` means no timeout, so virtually an infinite wait from the server.
    #[serde(with = "opt_ms", default, skip_serializing_if = "Option::is_none")]
    #[ruma_api(query)]
    pub timeout: Option<Duration>,

    /// Controls whether the client is automatically marked as online by polling this API.
    ///
    /// Defaults to `PresenceState::Online`.
    #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
    #[ruma_api(query)]
    pub set_presence: PresenceState,

    /// Lists of rooms we are interested by, represented by ranges.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lists: BTreeMap<String, request::List>,

    /// Specific rooms we are interested by.
    ///
    /// It is useful to receive updates from rooms that are possibly
    /// out-of-range of all the lists (see [`Self::lists`]).
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
    use ruma_common::{directory::RoomTypeFilter, serde::deserialize_cow_str, RoomId};
    use ruma_events::tag::TagName;
    use serde::de::Error as _;

    use super::{BTreeMap, Deserialize, OwnedRoomId, Serialize, StateEventType, UInt};

    /// A sliding sync list request (see [`super::Request::lists`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct List {
        /// The ranges of rooms we're interested in.
        pub ranges: Vec<(UInt, UInt)>,

        /// The details to be included per room.
        #[serde(flatten)]
        pub room_details: RoomDetails,

        /// Filters to apply to the list before sorting.
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

        /// Whether to return only encrypted rooms (as determined by the existence of an
        /// `m.room.encryption` state event), only unencrypted rooms, or both.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_encrypted: Option<bool>,

        /// Whether to return only invited rooms, only joined rooms, or both.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_invite: Option<bool>,

        /// Only list rooms with these create-types, or all.
        ///
        /// If a room type is specified in both `room_types` and `not_room_types`,
        /// `not_room_types` wins and the corresponding rooms are not included.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub room_types: Vec<RoomTypeFilter>,

        /// Only list rooms that are not of these create-types, or all.
        ///
        /// If a room type is specified in both `room_types` and `not_room_types`,
        /// `not_room_types` wins and the corresponding rooms are not included.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub not_room_types: Vec<RoomTypeFilter>,

        /// Filter a room based on its room tags.
        ///
        /// If multiple tags are present, a room can match any of the tags.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub tags: Vec<TagName>,

        /// Filter a room based on its room tags.
        ///
        /// Takes priority over `tags`. If a tag is listed in both `tags` and `not_tags`,
        /// `not_tags` wins and the corresponding rooms are not included.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub not_tags: Vec<TagName>,

        /// Filter a room based on the space it belongs to according to m.space.child events.
        ///
        /// If multiple spaces are present, a room can be part of any one of the listed spaces. The
        /// server will not navigate subspaces. The client must give a complete list of spaces to
        /// navigate.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        pub spaces: Vec<OwnedRoomId>,
    }

    /// Sliding sync request room subscription (see [`super::Request::room_subscriptions`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RoomSubscription {
        /// Required state for each returned room. An array of event type and
        /// state key tuples.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub required_state: Vec<(StateEventType, String)>,

        /// The maximum number of timeline events to return per room.
        pub timeline_limit: UInt,
    }

    /// Sliding sync request room details (see [`List::room_details`]).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct RoomDetails {
        /// Required state for each returned room. An array of event type and state key tuples.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub required_state: Vec<(StateEventType, String)>,

        /// The maximum number of timeline events to return per room.
        pub timeline_limit: UInt,
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

        /// Extensions may add further fields to the list.
        #[serde(flatten)]
        other: BTreeMap<String, serde_json::Value>,
    }

    impl Extensions {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.to_device.is_empty()
                && self.e2ee.is_empty()
                && self.account_data.is_empty()
                && self.receipts.is_empty()
                && self.typing.is_empty()
                && self.other.is_empty()
        }
    }

    /// Single entry for a room subscription configuration in an extension request.
    #[derive(Clone, Debug, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub enum ExtensionRoomConfig {
        /// Apply extension to all global room subscriptions.
        AllSubscribed,

        /// Additionally apply extension to this specific room.
        Room(OwnedRoomId),
    }

    impl Serialize for ExtensionRoomConfig {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::AllSubscribed => serializer.serialize_str("*"),
                Self::Room(r) => r.serialize(serializer),
            }
        }
    }

    impl<'de> Deserialize<'de> for ExtensionRoomConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            match deserialize_cow_str(deserializer)?.as_ref() {
                "*" => Ok(Self::AllSubscribed),
                other => Ok(Self::Room(RoomId::parse(other).map_err(D::Error::custom)?.to_owned())),
            }
        }
    }

    /// To-device messages extension.
    ///
    /// According to [MSC3885](https://github.com/matrix-org/matrix-spec-proposals/pull/3885).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ToDevice {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// Maximum number of to-device messages per response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,

        /// Give messages since this token only.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub since: Option<String>,
    }

    impl ToDevice {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none() && self.limit.is_none() && self.since.is_none()
        }
    }

    /// E2EE extension configuration.
    ///
    /// According to [MSC3884](https://github.com/matrix-org/matrix-spec-proposals/pull/3884).
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct E2EE {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,
    }

    impl E2EE {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Account-data extension .
    ///
    /// Not yet part of the spec proposal. Taken from the reference implementation
    /// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/account_data.go>
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct AccountData {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// List of list names for which account data should be enabled.
        ///
        /// This is specific to room account data (e.g. user-defined room tags).
        ///
        /// If not defined, will be enabled for *all* the lists appearing in the
        /// request. If defined and empty, will be disabled for all the lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// List of room names for which account data should be enabled.
        ///
        /// This is specific to room account data (e.g. user-defined room tags).
        ///
        /// If not defined, will be enabled for *all* the rooms appearing in the
        /// room subscriptions. If defined and empty, will be disabled for all
        /// the rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<ExtensionRoomConfig>>,
    }

    impl AccountData {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Receipt extension.
    ///
    /// According to [MSC3960](https://github.com/matrix-org/matrix-spec-proposals/pull/3960)
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Receipts {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// List of list names for which receipts should be enabled.
        ///
        /// If not defined, will be enabled for *all* the lists appearing in the
        /// request. If defined and empty, will be disabled for all the lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// List of room names for which receipts should be enabled.
        ///
        /// If not defined, will be enabled for *all* the rooms appearing in the
        /// room subscriptions. If defined and empty, will be disabled for all
        /// the rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<ExtensionRoomConfig>>,
    }

    impl Receipts {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Typing extension configuration.
    ///
    /// Not yet part of the spec proposal. Taken from the reference implementation
    /// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/typing.go>
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Typing {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// List of list names for which typing notifications should be enabled.
        ///
        /// If not defined, will be enabled for *all* the lists appearing in the
        /// request. If defined and empty, will be disabled for all the lists.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub lists: Option<Vec<String>>,

        /// List of room names for which typing notifications should be enabled.
        ///
        /// If not defined, will be enabled for *all* the rooms appearing in the
        /// room subscriptions. If defined and empty, will be disabled for all
        /// the rooms.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rooms: Option<Vec<ExtensionRoomConfig>>,
    }

    impl Typing {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none()
        }
    }

    /// Thread subscriptions extension.
    ///
    /// Specified as part of [MSC4308](https://github.com/matrix-org/matrix-spec-proposals/pull/4308).
    #[cfg(feature = "unstable-msc4308")]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ThreadSubscriptions {
        /// Activate or deactivate this extension.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub enabled: Option<bool>,

        /// Maximum number of thread subscription changes to receive in the response.
        ///
        /// Defaults to 100.
        /// Servers may impose a smaller limit than what is requested here.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub limit: Option<UInt>,
    }

    #[cfg(feature = "unstable-msc4308")]
    impl ThreadSubscriptions {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.enabled.is_none() && self.limit.is_none()
        }
    }
}

/// Response type for the `/sync` endpoint.
#[response(error = crate::Error)]
pub struct Response {
    /// Matches the `txn_id` sent by the request (see [`Request::txn_id`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_id: Option<String>,

    /// The token to supply in the `pos` parameter of the next `/sync` request
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
            txn_id: None,
            pos,
            lists: Default::default(),
            rooms: Default::default(),
            extensions: Default::default(),
        }
    }
}

/// HTTP types related to a [`Response`].
pub mod response {
    use ruma_common::OneTimeKeyAlgorithm;
    #[cfg(feature = "unstable-msc4308")]
    use ruma_common::OwnedEventId;
    use ruma_events::{
        receipt::SyncReceiptEvent, typing::SyncTypingEvent, AnyGlobalAccountDataEvent,
        AnyRoomAccountDataEvent, AnyStrippedStateEvent, AnyToDeviceEvent,
    };

    use super::{
        super::DeviceLists, AnySyncStateEvent, AnySyncTimelineEvent, BTreeMap, Deserialize,
        JsOption, OwnedMxcUri, OwnedRoomId, OwnedUserId, Raw, Serialize, UInt,
        UnreadNotificationsCount,
    };
    #[cfg(feature = "unstable-msc4308")]
    use crate::threads::get_thread_subscriptions_changes::unstable::{
        ThreadSubscription, ThreadUnsubscription,
    };

    /// A sliding sync response updates to joiend rooms (see
    /// [`super::Response::lists`]).
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct List {
        /// The total number of rooms found for this list.
        pub count: UInt,
    }

    /// A sliding sync response updated room (see [`super::Response::rooms`]).
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Room {
        /// The name as calculated by the server.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        /// The avatar.
        #[serde(default, skip_serializing_if = "JsOption::is_undefined")]
        pub avatar: JsOption<OwnedMxcUri>,

        /// Whether it is an initial response.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub initial: Option<bool>,

        /// Whether it is a direct room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_dm: Option<bool>,

        /// If this is `Some(_)`, this is a not-yet-accepted invite containing
        /// the given stripped state events.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub invite_state: Option<Vec<Raw<AnyStrippedStateEvent>>>,

        /// Number of unread notifications.
        #[serde(flatten, default, skip_serializing_if = "UnreadNotificationsCount::is_empty")]
        pub unread_notifications: UnreadNotificationsCount,

        /// Message-like events and live state events.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub timeline: Vec<Raw<AnySyncTimelineEvent>>,

        /// State events as configured by the request.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub required_state: Vec<Raw<AnySyncStateEvent>>,

        /// The `prev_batch` allowing you to paginate through the messages
        /// before the given ones.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,

        /// True if the number of events returned was limited by the limit on
        /// the filter.
        #[serde(default, skip_serializing_if = "ruma_common::serde::is_default")]
        pub limited: bool,

        /// The number of users with membership of `join`, including the
        /// client’s own user ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub joined_count: Option<UInt>,

        /// The number of users with membership of `invite`.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub invited_count: Option<UInt>,

        /// The number of timeline events which have just occurred and are not
        /// historical.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub num_live: Option<UInt>,

        /// The bump stamp of the room.
        ///
        /// It can be interpreted as a “recency stamp” or “streaming order
        /// index”. For example, consider `roomA` with `bump_stamp = 2`, `roomB`
        /// with `bump_stamp = 1` and `roomC` with `bump_stamp = 0`. If `roomC`
        /// receives an update, its `bump_stamp` will be 3.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bump_stamp: Option<UInt>,

        /// Heroes of the room.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub heroes: Option<Vec<Hero>>,
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

        /// The name.
        #[serde(rename = "displayname", skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        /// The avatar.
        #[serde(rename = "avatar_url", skip_serializing_if = "Option::is_none")]
        pub avatar: Option<OwnedMxcUri>,
    }

    impl Hero {
        /// Creates a new `Hero` with the given user ID.
        pub fn new(user_id: OwnedUserId) -> Self {
            Self { user_id, name: None, avatar: None }
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
        #[serde(default, skip_serializing_if = "E2EE::is_empty")]
        pub e2ee: E2EE,

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
    }

    impl Extensions {
        /// Whether the extension data is empty.
        ///
        /// True if neither to-device, e2ee nor account data are to be found.
        pub fn is_empty(&self) -> bool {
            self.to_device.is_none()
                && self.e2ee.is_empty()
                && self.account_data.is_empty()
                && self.receipts.is_empty()
                && self.typing.is_empty()
        }
    }

    /// To-device extension response.
    ///
    /// According to [MSC3885](https://github.com/matrix-org/matrix-spec-proposals/pull/3885).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct ToDevice {
        /// Fetch the next batch from this entry.
        pub next_batch: String,

        /// The to-device events.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub events: Vec<Raw<AnyToDeviceEvent>>,
    }

    /// E2EE extension response.
    ///
    /// According to [MSC3884](https://github.com/matrix-org/matrix-spec-proposals/pull/3884).
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct E2EE {
        /// Information on E2EE device updates.
        #[serde(default, skip_serializing_if = "DeviceLists::is_empty")]
        pub device_lists: DeviceLists,

        /// For each key algorithm, the number of unclaimed one-time keys
        /// currently held on the server for a device.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub device_one_time_keys_count: BTreeMap<OneTimeKeyAlgorithm, UInt>,

        /// The unused fallback key algorithms.
        ///
        /// The presence of this field indicates that the server supports
        /// fallback keys.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub device_unused_fallback_key_types: Option<Vec<OneTimeKeyAlgorithm>>,
    }

    impl E2EE {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.device_lists.is_empty()
                && self.device_one_time_keys_count.is_empty()
                && self.device_unused_fallback_key_types.is_none()
        }
    }

    /// Account-data extension response .
    ///
    /// Not yet part of the spec proposal. Taken from the reference implementation
    /// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/account_data.go>
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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.global.is_empty() && self.rooms.is_empty()
        }
    }

    /// Receipt extension response.
    ///
    /// According to [MSC3960](https://github.com/matrix-org/matrix-spec-proposals/pull/3960)
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Receipts {
        /// The ephemeral receipt room event for each room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, Raw<SyncReceiptEvent>>,
    }

    impl Receipts {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.rooms.is_empty()
        }
    }

    /// Typing extension response.
    ///
    /// Not yet part of the spec proposal. Taken from the reference implementation
    /// <https://github.com/matrix-org/sliding-sync/blob/main/sync3/extensions/typing.go>
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    pub struct Typing {
        /// The ephemeral typing event for each room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        pub rooms: BTreeMap<OwnedRoomId, Raw<SyncTypingEvent>>,
    }

    impl Typing {
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.rooms.is_empty()
        }
    }

    /// Thread subscriptions extension response.
    ///
    /// Specified as part of [MSC4308](https://github.com/matrix-org/matrix-spec-proposals/pull/4308).
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
        /// Whether all fields are empty or `None`.
        pub fn is_empty(&self) -> bool {
            self.subscribed.is_empty() && self.unsubscribed.is_empty() && self.prev_batch.is_none()
        }
    }
}

#[cfg(test)]
mod tests {
    use ruma_common::owned_room_id;

    use super::request::ExtensionRoomConfig;

    #[test]
    fn serialize_request_extension_room_config() {
        let entry = ExtensionRoomConfig::AllSubscribed;
        assert_eq!(serde_json::to_string(&entry).unwrap().as_str(), r#""*""#);

        let entry = ExtensionRoomConfig::Room(owned_room_id!("!foo:bar.baz"));
        assert_eq!(serde_json::to_string(&entry).unwrap().as_str(), r#""!foo:bar.baz""#);
    }

    #[test]
    fn deserialize_request_extension_room_config() {
        assert_eq!(
            serde_json::from_str::<ExtensionRoomConfig>(r#""*""#).unwrap(),
            ExtensionRoomConfig::AllSubscribed
        );

        assert_eq!(
            serde_json::from_str::<ExtensionRoomConfig>(r#""!foo:bar.baz""#).unwrap(),
            ExtensionRoomConfig::Room(owned_room_id!("!foo:bar.baz"))
        );
    }
}
