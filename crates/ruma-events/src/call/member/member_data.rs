//! Types for MatrixRTC `m.call.member` state event content data ([MSC3401])
//!
//! [MSC3401]: https://github.com/matrix-org/matrix-spec-proposals/pull/3401

use std::time::Duration;

use as_variant::as_variant;
use ruma_common::MilliSecondsSinceUnixEpoch;
use ruma_macros::StringEnum;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::focus::{ActiveFocus, ActiveLivekitFocus, Focus};
use crate::PrivOwnedStr;

/// The data object that contains the information for one membership.
///
/// It can be a legacy or a normal MatrixRTC Session membership.
///
/// The legacy format contains time information to compute if it is expired or not.
/// SessionMembershipData does not have the concept of timestamp based expiration anymore.
/// The state event will reliably be set to empty when the user disconnects.
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub enum MembershipData<'a> {
    /// The legacy format (using an array of memberships for each device -> one event per user)
    Legacy(&'a LegacyMembershipData),
    /// One event per device. `SessionMembershipData` contains all the information required to
    /// represent the current membership state of one device.
    Session(&'a SessionMembershipData),
}

impl<'a> MembershipData<'a> {
    /// The application this RTC membership participates in (the session type, can be `m.call`...)
    pub fn application(&self) -> &Application {
        match self {
            MembershipData::Legacy(data) => &data.application,
            MembershipData::Session(data) => &data.application,
        }
    }

    /// The device id of this membership.
    pub fn device_id(&self) -> &String {
        match self {
            MembershipData::Legacy(data) => &data.device_id,
            MembershipData::Session(data) => &data.device_id,
        }
    }

    /// The active focus is a FocusType specific object that describes how this user
    /// is currently connected.
    ///
    /// It can use the foci_preferred list to choose one of the available (preferred)
    /// foci or specific information on how to connect to this user.
    ///
    /// Every user needs to converge to use the same focus_active type.
    pub fn focus_active(&self) -> &ActiveFocus {
        match self {
            MembershipData::Legacy(_) => &ActiveFocus::Livekit(ActiveLivekitFocus {
                focus_select: super::focus::FocusSelection::OldestMembership,
            }),
            MembershipData::Session(data) => &data.focus_active,
        }
    }

    /// The list of available/preferred options this user provides to connect to the call.
    pub fn foci_preferred(&self) -> &Vec<Focus> {
        match self {
            MembershipData::Legacy(data) => &data.foci_active,
            MembershipData::Session(data) => &data.foci_preferred,
        }
    }

    /// The application of the membership is "m.call" and the scope is "m.room".
    pub fn is_room_call(&self) -> bool {
        as_variant!(self.application(), Application::Call)
            .is_some_and(|call| call.scope == CallScope::Room)
    }

    /// The application of the membership is "m.call".
    pub fn is_call(&self) -> bool {
        as_variant!(self.application(), Application::Call).is_some()
    }

    /// Checks if the event is expired. This is only relevant for LegacyMembershipData
    /// returns `false` if its SessionMembershipData
    pub fn is_expired(&self, origin_server_ts: Option<MilliSecondsSinceUnixEpoch>) -> bool {
        match self {
            MembershipData::Legacy(data) => data.is_expired(origin_server_ts),
            MembershipData::Session(_) => false,
        }
    }

    /// Gets the created_ts of the event.
    ///
    /// This is the `origin_server_ts` for session data.
    /// For legacy events this can either be the origin server ts or a copy from the
    /// `origin_server_ts` since we expect legacy events to get updated (when a new device
    /// joins/leaves).
    pub fn created_ts(&self) -> Option<MilliSecondsSinceUnixEpoch> {
        match self {
            MembershipData::Legacy(data) => data.created_ts,
            MembershipData::Session(data) => data.created_ts,
        }
    }
}

/// A membership describes one of the sessions this user currently partakes.
///
/// The application defines the type of the session.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LegacyMembershipData {
    /// The type of the MatrixRTC session the membership belongs to.
    ///
    /// e.g. call, spacial, document...
    #[serde(flatten)]
    pub application: Application,

    /// The device id of this membership.
    ///
    /// The same user can join with their phone/computer.
    pub device_id: String,

    /// The duration in milliseconds relative to the time this membership joined
    /// during which the membership is valid.
    ///
    /// The time a member has joined is defined as:
    /// `MIN(content.created_ts, event.origin_server_ts)`
    #[serde(with = "ruma_common::serde::duration::ms")]
    pub expires: Duration,

    /// Stores a copy of the `origin_server_ts` of the initial session event.
    ///
    /// If the membership is updated this field will be used to track to
    /// original `origin_server_ts`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_ts: Option<MilliSecondsSinceUnixEpoch>,

    /// A list of the foci in use for this membership.
    pub foci_active: Vec<Focus>,

    /// The id of the membership.
    ///
    /// This is required to guarantee uniqueness of the event.
    /// Sending the same state event twice to synapse makes the HS drop the second one and return
    /// 200.
    #[serde(rename = "membershipID")]
    pub membership_id: String,
}

impl LegacyMembershipData {
    /// Checks if the event is expired.
    ///
    /// Defaults to using `created_ts` of the [`LegacyMembershipData`].
    /// If no `origin_server_ts` is provided and the event does not contain `created_ts`
    /// the event will be considered as not expired.
    /// In this case, a warning will be logged.
    ///
    /// # Arguments
    ///
    /// * `origin_server_ts` - a fallback if [`LegacyMembershipData::created_ts`] is not present
    pub fn is_expired(&self, origin_server_ts: Option<MilliSecondsSinceUnixEpoch>) -> bool {
        let ev_created_ts = self.created_ts.or(origin_server_ts);

        if let Some(ev_created_ts) = ev_created_ts {
            let now = MilliSecondsSinceUnixEpoch::now().to_system_time();
            let expire_ts = ev_created_ts.to_system_time().map(|t| t + self.expires);
            now > expire_ts
        } else {
            // This should not be reached since we only allow events that have copied over
            // the origin server ts. `set_created_ts_if_none`
            warn!("Encountered a Call Member state event where the origin_ts (or origin_server_ts) could not be found.\
            It is treated as a non expired event but this might be wrong.");
            false
        }
    }
}

/// Initial set of fields of [`LegacyMembershipData`].
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct LegacyMembershipDataInit {
    /// The type of the MatrixRTC session the membership belongs to.
    ///
    /// e.g. call, spacial, document...
    pub application: Application,

    /// The device id of this membership.
    ///
    /// The same user can join with their phone/computer.
    pub device_id: String,

    /// The duration in milliseconds relative to the time this membership joined
    /// during which the membership is valid.
    ///
    /// The time a member has joined is defined as:
    /// `MIN(content.created_ts, event.origin_server_ts)`
    pub expires: Duration,

    /// A list of the focuses (foci) in use for this membership.
    pub foci_active: Vec<Focus>,

    /// The id of the membership.
    ///
    /// This is required to guarantee uniqueness of the event.
    /// Sending the same state event twice to synapse makes the HS drop the second one and return
    /// 200.
    pub membership_id: String,
}

impl From<LegacyMembershipDataInit> for LegacyMembershipData {
    fn from(init: LegacyMembershipDataInit) -> Self {
        let LegacyMembershipDataInit {
            application,
            device_id,
            expires,
            foci_active,
            membership_id,
        } = init;
        Self { application, device_id, expires, created_ts: None, foci_active, membership_id }
    }
}

/// Stores all the information for a MatrixRTC membership. (one for each device)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct SessionMembershipData {
    /// The type of the MatrixRTC session the membership belongs to.
    ///
    /// e.g. call, spacial, document...
    #[serde(flatten)]
    pub application: Application,

    /// The device id of this membership.
    ///
    /// The same user can join with their phone/computer.
    pub device_id: String,

    /// A list of the foci that this membership proposes to use.
    pub foci_preferred: Vec<Focus>,

    /// Data required to determine the currently used focus by this member.
    pub focus_active: ActiveFocus,

    /// Stores a copy of the `origin_server_ts` of the initial session event.
    ///
    /// This is not part of the serialized event and computed after serialization.
    #[serde(skip)]
    pub created_ts: Option<MilliSecondsSinceUnixEpoch>,
}

/// The type of the MatrixRTC session.
///
/// This is not the application/client used by the user but the
/// type of MatrixRTC session e.g. calling (`m.call`), third-room, whiteboard could be
/// possible applications.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "application")]
pub enum Application {
    /// The rtc application (session type) for VoIP call.
    #[serde(rename = "m.call")]
    Call(CallApplicationContent),
}

/// Call specific parameters of a `m.call.member` event.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CallApplicationContent {
    /// An identifier for calls.
    ///
    /// All members using the same `call_id` will end up in the same call.
    ///
    /// Does not need to be a uuid.
    ///
    /// `""` is used for room scoped calls.
    pub call_id: String,

    /// Who owns/joins/controls (can modify) the call.
    pub scope: CallScope,
}

impl CallApplicationContent {
    /// Initialize a [`CallApplicationContent`].
    ///
    /// # Arguments
    ///
    /// * `call_id` - An identifier for calls. All members using the same `call_id` will end up in
    ///   the same call. Does not need to be a uuid. `""` is used for room scoped calls.
    /// * `scope` - Who owns/joins/controls (can modify) the call.
    pub fn new(call_id: String, scope: CallScope) -> Self {
        Self { call_id, scope }
    }
}

/// The call scope defines different call ownership models.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_enum(rename_all = "m.snake_case")]
pub enum CallScope {
    /// A call which every user of a room can join and create.
    ///
    /// There is no particular name associated with it.
    ///
    /// There can only be one per room.
    Room,

    /// A user call is owned by a user.
    ///
    /// Each user can create one there can be multiple per room. They are started and ended by the
    /// owning user.
    User,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
