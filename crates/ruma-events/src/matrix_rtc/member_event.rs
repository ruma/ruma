//! Types for matrixRTC state events ([MSC3401]).
//!
//! This implements a newer/updated version of MSC3401.
//!
//! [MSC3401]: https://github.com/matrix-org/matrix-spec-proposals/pull/3401

use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedUserId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::{
    focus::{ActiveFocus, Focus},
    member_data::{Application, LegacyMembershipData, MembershipData, SessionMembershipData},
};
use crate::{PossiblyRedactedStateEventContent, StateEventType};

/// The member state event for a matrixRTC session.
///
/// This is the object containing all the data related to a matrix users participation in a
/// matrixRTC session.
///
/// This is a unit struct with the enum [`MemberEventContent`] because a Ruma state event cannot be
/// an enum and we need this to be an untagged enum for parsing purposes. (see
/// [`MemberEventContent`])
///
/// This struct also exposes allows to call the methods from [`MemberEventContent`].
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[ruma_event(type = "org.matrix.msc3401.call.member", kind = State, state_key_type = OwnedUserId, custom_possibly_redacted)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct CallMemberEventContent(pub MemberEventContent);

impl CallMemberEventContent {
    /// Creates a new [`CallMemberEventContent`] with [`LegacyMembershipData`].
    pub fn new_legacy(memberships: Vec<LegacyMembershipData>) -> Self {
        CallMemberEventContent(MemberEventContent::new_legacy(memberships))
    }

    /// Creates a new [`CallMemberEventContent`] with [`SessionMembershipData`].
    pub fn new(
        application: Application,
        device_id: String,
        focus_active: ActiveFocus,
        foci_preferred: Vec<Focus>,
        created_ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Self {
        CallMemberEventContent(MemberEventContent::new(
            application,
            device_id,
            focus_active,
            foci_preferred,
            created_ts,
        ))
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
    ) -> Vec<MembershipData> {
        self.0.active_memberships(origin_server_ts)
    }
    /// All the memberships for this event. Can only contain multiple elements in the case of legacy
    /// `m.call.member` state events.
    pub fn memberships(&self) -> Vec<MembershipData> {
        self.0.memberships()
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
        self.0.set_created_ts_if_none(origin_server_ts);
    }
}

/// The PossiblyRedacted version of [`CallMemberEventContent`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct PossiblyRedactedCallMemberEventContent(pub MemberEventContent);

impl ruma_events::content::EventContent for PossiblyRedactedCallMemberEventContent {
    type EventType = StateEventType;
    fn event_type(&self) -> Self::EventType {
        StateEventType::CallMember
    }
}

impl PossiblyRedactedStateEventContent for PossiblyRedactedCallMemberEventContent {
    type StateKey = String;
}

/// The object containing the actual content of a [`CallMemberEventContent`].
///
/// There are two versions of [`CallMemberEventContent`]. One for legacy events,
/// which contain an array of memberships and one version for the new format that just
/// has one membership.
///
/// This is an untagged serde enum to be able to parse both versions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(untagged)]
pub enum MemberEventContent {
    /// The legacy format for m.call.member events. (An array of memberships. The devices of one
    /// user.)
    LegacyContent(LegacyMembershipContent),
    /// Normal membership events. One event per membership. Multiple state keys will
    /// be used to describe multiple devices for one user.
    SessionContent(SessionMembershipData),
    /// An empty content means this user has been in a rtc session but is not anymore.
    Empty {},
}

impl MemberEventContent {
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
    ) -> Vec<MembershipData> {
        match self {
            MemberEventContent::LegacyContent(content) => content
                .memberships
                .clone()
                .into_iter()
                .filter(|m| !m.is_expired(origin_server_ts))
                .map(MembershipData::Legacy)
                .collect(),
            MemberEventContent::SessionContent(content) => {
                [MembershipData::Session(content.clone())].to_vec()
            }
            MemberEventContent::Empty {} => Vec::new(),
        }
    }
    /// All the memberships for this event. Can only contain multiple elements in the case of legacy
    /// `m.call.member` state events.
    pub fn memberships(&self) -> Vec<MembershipData> {
        match self {
            MemberEventContent::LegacyContent(content) => {
                content.memberships.clone().into_iter().map(MembershipData::Legacy).collect()
            }
            MemberEventContent::SessionContent(content) => {
                [MembershipData::Session(content.clone())].to_vec()
            }
            MemberEventContent::Empty {} => Vec::new(),
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
            MemberEventContent::LegacyContent(content) => {
                content.memberships.iter_mut().for_each(|m: &mut LegacyMembershipData| {
                    m.created_ts.get_or_insert(origin_server_ts);
                });
            }
            MemberEventContent::SessionContent(m) => {
                m.created_ts.get_or_insert(origin_server_ts);
            }
            _ => (),
        }
    }
}

/// Legacy content with an array of memberships. See also: [`MemberEventContent`]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct LegacyMembershipContent {
    /// A list of all the memberships that user currently has in this room.
    ///
    /// There can be multiple ones in cases the user participates with multiple devices or there
    /// are multiple RTC applications running.
    ///
    /// e.g. a call and a spacial experience.
    ///
    /// Important: This includes expired memberships.
    /// To retrieve a list including only valid memberships,
    /// see [`active_memberships`](CallMemberEventContent::active_memberships).
    memberships: Vec<LegacyMembershipData>,
}
