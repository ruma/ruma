//! Types for matrixRTC state events ([MSC3401]).
//!
//! This implements a newer/updated version of MSC3401.
//!
//! [MSC3401]: https://github.com/matrix-org/matrix-spec-proposals/pull/3401

use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedUserId};
use ruma_macros::{EventContent, StringEnum};
use serde::{Deserialize, Serialize};

use super::{
    focus::{ActiveFocus, Focus},
    member_data::{Application, LegacyMembershipData, MembershipData, SessionMembershipData},
};
use crate::{
    PossiblyRedactedStateEventContent, PrivOwnedStr, RedactContent, RedactedStateEventContent,
    StateEventType,
};

/// The member state event for a matrixRTC session.
///
/// This is the object containing all the data related to a matrix users participation in a
/// matrixRTC session.
///
/// This is a unit struct with the enum [`CallMemberEventContent`] because a Ruma state event cannot
/// be an enum and we need this to be an untagged enum for parsing purposes. (see
/// [`CallMemberEventContent`])
///
/// This struct also exposes allows to call the methods from [`CallMemberEventContent`].
#[derive(Clone, Debug, Serialize, Deserialize, EventContent, PartialEq)]
#[ruma_event(type = "org.matrix.msc3401.call.member", kind = State, state_key_type = OwnedUserId, custom_redacted, custom_possibly_redacted)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum CallMemberEventContent {
    /// The legacy format for m.call.member events. (An array of memberships. The devices of one
    /// user.)
    LegacyContent(LegacyMembershipContent),
    /// Normal membership events. One event per membership. Multiple state keys will
    /// be used to describe multiple devices for one user.
    SessionContent(SessionMembershipData),
    /// An empty content means this user has been in a rtc session but is not anymore.
    Empty {
        /// An empty call member state event can optionally contain a leave reason.
        /// If it is `None` the user has left the call ordinary. (Intentional hangup)
        #[serde(skip_serializing_if = "Option::is_none")]
        leave_reason: Option<LeaveReason>,
    },
}

impl CallMemberEventContent {
    /// Creates a new [`CallMemberEventContent`] with [`LegacyMembershipData`].
    pub fn new_legacy(memberships: Vec<LegacyMembershipData>) -> Self {
        Self::LegacyContent(LegacyMembershipContent {
            memberships, //: memberships.into_iter().map(MembershipData::Legacy).collect(),
        })
    }

    /// Creates a new [`CallMemberEventContent`] with [`SessionMembershipData`].
    pub fn new(
        application: Application,
        device_id: String,
        focus_active: ActiveFocus,
        foci_preferred: Vec<Focus>,
        created_ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Self {
        Self::SessionContent(SessionMembershipData {
            application,
            device_id,
            focus_active,
            foci_preferred,
            created_ts,
        })
    }

    /// Creates a new Empty [`CallMemberEventContent`] representing a left membership.
    pub fn new_empty(leave_reason: Option<LeaveReason>) -> Self {
        Self::Empty { leave_reason }
    }
    /// All non expired memberships in this member event.
    ///
    /// In most cases you want to use this method instead of the public memberships field.
    /// The memberships field will also include expired events.
    ///
    /// This copies all the memberships and converts them
    /// # Arguments
    ///
    /// * `origin_server_ts` - optionally the `origin_server_ts` can be passed as a fallback in the
    ///   Membership does not contain [`LegacyMembershipData::created_ts`]. (`origin_server_ts` will
    ///   be ignored if [`LegacyMembershipData::created_ts`] is `Some`)
    pub fn active_memberships(
        &self,
        origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Vec<MembershipData<'_>> {
        match self {
            CallMemberEventContent::LegacyContent(content) => content
                .memberships
                .iter()
                .filter(|m| !m.is_expired(origin_server_ts))
                .map(MembershipData::Legacy)
                .collect(),
            CallMemberEventContent::SessionContent(content) => {
                [content].map(MembershipData::Session).to_vec()
            }
            CallMemberEventContent::Empty { leave_reason: _ } => Vec::new(),
        }
    }

    /// All the memberships for this event. Can only contain multiple elements in the case of legacy
    /// `m.call.member` state events.
    pub fn memberships(&self) -> Vec<MembershipData<'_>> {
        match self {
            CallMemberEventContent::LegacyContent(content) => {
                content.memberships.iter().map(MembershipData::Legacy).collect()
            }
            CallMemberEventContent::SessionContent(content) => {
                [content].map(MembershipData::Session).to_vec()
            }
            CallMemberEventContent::Empty { leave_reason: _ } => Vec::new(),
        }
    }

    /// Set the `created_ts` of each [`MembershipData::Legacy`] in this event.
    ///
    /// Each call member event contains the `origin_server_ts` and `content.create_ts`.
    /// `content.create_ts` is undefined for the initial event of a session (because the
    /// `origin_server_ts` is not known on the client).
    /// In the rust sdk we want to copy over the `origin_server_ts` of the event into the
    /// (This allows to use `MinimalStateEvents` and still be able to determine if a
    /// expired)
    pub fn set_created_ts_if_none(&mut self, origin_server_ts: MilliSecondsSinceUnixEpoch) {
        match self {
            CallMemberEventContent::LegacyContent(content) => {
                content.memberships.iter_mut().for_each(|m: &mut LegacyMembershipData| {
                    m.created_ts.get_or_insert(origin_server_ts);
                });
            }
            CallMemberEventContent::SessionContent(m) => {
                m.created_ts.get_or_insert(origin_server_ts);
            }
            _ => (),
        }
    }
}

impl RedactContent for CallMemberEventContent {
    type Redacted = RedactedCallMemberEventContent;

    fn redact(self, _version: &ruma_common::RoomVersionId) -> Self::Redacted {
        RedactedCallMemberEventContent {}
    }
}

/// The PossiblyRedacted version of [`CallMemberEventContent`].
///
/// Since [`CallMemberEventContent`] has the Empty {} state it already is compatible
/// with the redacted version of the state event content.
pub type PossiblyRedactedCallMemberEventContent = CallMemberEventContent;

impl PossiblyRedactedStateEventContent for PossiblyRedactedCallMemberEventContent {
    type StateKey = OwnedUserId;
}

/// The Redacted version of [`CallMemberEventContent`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct RedactedCallMemberEventContent {}

impl ruma_events::content::EventContent for RedactedCallMemberEventContent {
    type EventType = StateEventType;
    fn event_type(&self) -> Self::EventType {
        StateEventType::CallMember
    }
}

impl RedactedStateEventContent for RedactedCallMemberEventContent {
    type StateKey = OwnedUserId;
}

/// This is the optional value for an empty membership event content:
/// [`CallMemberEventContent::Empty`]. It is used when the user disconnected and a Future ([MSC4140](https://github.com/matrix-org/matrix-spec-proposals/pull/4140))
/// was used to update the after the client was not reachable anymore.
#[derive(Clone, PartialEq, StringEnum)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_enum(rename_all = "m.snake_case")]
pub enum LeaveReason {
    /// The user left the call by loosing network connection or closing
    /// the client before it was able to send the leave event.
    LostConnection,
    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// Legacy content with an array of memberships. See also: [`CallMemberEventContent`]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LegacyMembershipContent {
    /// A list of all the memberships that user currently has in this room.
    ///
    /// There can be multiple ones in case the user participates with multiple devices or there
    /// are multiple RTC applications running.
    ///
    /// e.g. a call and a spacial experience.
    ///
    /// Important: This includes expired memberships.
    /// To retrieve a list including only valid memberships,
    /// see [`active_memberships`](CallMemberEventContent::active_memberships).
    memberships: Vec<LegacyMembershipData>,
}
