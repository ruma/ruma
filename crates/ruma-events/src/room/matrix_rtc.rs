//! Types for matrix rtc state events ([MSC3401]).
//!
//! [MSC3927]: https://github.com/matrix-org/matrix-spec-proposals/pull/3401

use ruma_common::{MilliSecondsSinceUnixEpoch, OwnedUserId};
use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// The member state event for a matrixRTC session
///
/// This is the object containing all the data related to a matrix users participation in a
/// matrixRTC session. It consists of memberships/sessions.
#[derive(Clone, Debug, Serialize, Deserialize, EventContent)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc3401.call.member", kind = State, state_key_type = OwnedUserId)]
pub struct CallMemberEventContent {
    ///A list of all the memberships that user currently has in this room.
    ///(can be multiple ones in cases the user participates with multiple devices or there are
    ///multiple RTC applications (e.g. a call and a spacial experience) running.)
    memberships: Vec<Membership>,
}

/// A membership describes one of the sessions this user currently partakes.
///
/// The application defines the type of the session.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct Membership {
    /// The type of the matrixRTC session the membership belongs to. (e.g. call, spacial,
    /// document...)
    #[serde(flatten)]
    pub application: Application,
    /// The device id of this membership. (The same user can join with their phone/computer)
    pub device_id: String,
    /// The duration in milliseconds relative to the time this membership joined
    /// (`MIN(content.created_ts, event.origin_server_ts)`) during which the membership is valid.
    pub expires: u64,
    /// Contains the `origin_server_ts` of the initial session join.
    /// If the membership is updated this field will be used to track to
    /// original `origin_server_ts`
    pub created_ts: Option<MilliSecondsSinceUnixEpoch>,
    /// A list of the foci in use for this membership
    pub foci_active: Vec<Foci>,
    /// The id of the membership. This is required to guarantee uniqueness of the event.
    /// (Sending the same state event twice to synapse makes the HS drop the second one and return
    /// 200)
    #[serde(rename = "membershipID")]
    pub membership_id: String,
}


/// Description of the SFU/Foci a membership can be connected to
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Foci {
    /// Livekit is one possible type of SFU/Foci that can be used for a matrixRTC session
    Livekit(LivekitFoci),
}

/// The fields to describe livekit as an `active_foci`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LivekitFoci {
    /// The alias where the livekit sessions can be reached
    #[serde(rename = "livekit_alias")]
    pub alias: String,
    /// The url of the jwt server of the used livekit instance
    #[serde(rename = "livekit_service_url")]
    pub service_url: String,
}

/// The type of the matrixRTC session.
/// (this is not the application/client used by the user but the
/// type of matrixRTC session e.g. calling, third-room, whiteboard could be
/// possible applications.)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "application")]
pub enum Application {
    #[serde(rename = "m.call")]
    /// A VoIP call
    Call(CallApplicationContent),
    /// Any other application that is not Specced
    Unknown(serde_json::Value),
}

/// Call specific parameters membership parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallApplicationContent {
    /// An optional identifier for calls. Only relevant for some calls.
    pub call_id: String,
    /// Who owns/joins/controls (can modify) the call.
    pub scope: CallScope,
}

/// The call scope defines different call ownership models.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CallScope {
    /// A call which every user of a room can join and create.
    /// there is no particular name associated with it.
    /// There can only be one per room.
    #[serde(rename = "m.room")]
    Room,
    /// A user call is owned by a user. Each user can create one
    /// there can be multiple per room. They are started and ended by
    /// the owning user.
    #[serde(rename = "m.user")]
    User,
}
