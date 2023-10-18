//! Types for matrix rtc state events ([MSC3401]).
//!
//! [MSC3927]: https://github.com/matrix-org/matrix-spec-proposals/pull/3401

use std::time::Duration;

use as_variant::as_variant;
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

impl CallMemberEventContent {
    /// Creates a new `CallMemberEventContent`.
    pub fn new(memberships: Vec<Membership>) -> Self {
        Self { memberships }
    }

    /// All non expired memberships in this member event
    /// # Arguments
    /// - [`origin_server_ts`] optionally the `origin_server_ts` can be passed as a fallback in case
    ///   the Membership does not contain `created_ts`. (`origin_server_ts` will be ignored if
    ///   `created_ts` is `Some`)
    pub fn memberships(
        &self,
        origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
    ) -> Vec<&Membership> {
        self.memberships.iter().filter(|m| !m.is_expired(origin_server_ts)).collect()
    }

    /// All memberships in this event (including expired ones)
    pub fn memberships_including_expired(&self) -> Vec<&Membership> {
        self.memberships.iter().collect()
    }

    /// Each call member event contains the `origin_server_ts` and `content.create_ts`.
    /// `content.create_ts` is undefined for the initial event of a session (because the
    /// `origin_server_ts` is not known on the client).
    /// In the rust sdk we want to copy over the `origin_server_ts` of the event into the content.
    /// (This allows to use `MinimalStateEvents` and still be able to determine if a membership is
    /// expired)
    pub fn set_created_ts_if_none(&mut self, origin_server_ts: MilliSecondsSinceUnixEpoch) {
        self.memberships.iter_mut().for_each(|m| {
            m.created_ts = Some(m.created_ts.unwrap_or(origin_server_ts));
        });
    }
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

impl Membership {
    /// Application is "m.call" and scope is "m.room"
    pub fn is_room_call(&self) -> bool {
        as_variant!(&self.application, Application::Call)
            .is_some_and(|call| call.scope == CallScope::Room)
    }

    /// Application is "m.call"
    pub fn is_call(&self) -> bool {
        as_variant!(&self.application, Application::Call).is_some()
    }

    /// Check if the event is expired.
    /// Defaults to using `created_ts` in the event content.
    /// If no `origin_server_ts` is provided and the event does not contain `created_ts`
    /// the event will be considered as not expired. (A warning will be logged)
    /// # Arguments
    ///  - [`origin_server_ts`] a fallback if `created_ts` is not present
    pub fn is_expired(&self, origin_server_ts: Option<MilliSecondsSinceUnixEpoch>) -> bool {
        let ev_created_ts = match (self.created_ts, origin_server_ts) {
            (Some(created_ts), Some(_)) => Some(created_ts),
            (None, Some(server_ts)) => Some(server_ts),
            (Some(created_ts), None) => Some(created_ts),
            _ => None,
        };
        if let Some(ev_created_ts) = ev_created_ts {
            let now = MilliSecondsSinceUnixEpoch::now().to_system_time();
            let expire_ts =
                ev_created_ts.to_system_time().map(|t| t + Duration::from_millis(self.expires));
            now > expire_ts
        } else {
            // This should not be reached since we only allow events that have copied over
            // the origin server ts. `copy_origin_server_ts_to_membership`
            warn!("Encountered a Call Member state event where the origin_ts (or origin_server_ts) could not be found.
            It is treated as a non expired event but this might be wrong.");
            false
        }
    }
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::{
        Application, CallApplicationContent, CallMemberEventContent, CallScope, Foci, Membership,
    };

    fn _remove_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect::<String>()
    }
    fn create_call_member_event_content() -> CallMemberEventContent {
        CallMemberEventContent::new(vec![Membership {
            application: Application::Call(CallApplicationContent {
                call_id: "123456".to_owned(),
                scope: CallScope::Room,
            }),
            device_id: "ABCDE".to_owned(),
            expires: 36000,
            foci_active: vec![Foci {
                livekit_alias: "1".to_owned(),
                livekit_service_url: "https://livekit.com".to_owned(),
                foci_type: "livekit".to_owned(),
            }],
            membership_id: "0".to_owned(),
            created_ts: None,
        }])
    }
    #[test]
    fn serialize_call_member_event_content() {
        let call_member_event: &str = &_remove_whitespace(
            r#"
            {
                "memberships": [
                    {
                        "application": "m.call",
                        "call_id": "123456",
                        "scope": "m.room",
                        "device_id": "ABCDE",
                        "expires": 36000,
                        "foci_active": [
                            {
                                "livekit_alias": "1",
                                "livekit_service_url": "https://livekit.com",
                                "type": "livekit"
                            }
                        ],
                        "membershipID": "0"
                    }
                ]
            }
            "#,
        );
        println!("{}", serde_json::to_string_pretty(&create_call_member_event_content()).unwrap());

        assert_eq!(
            call_member_event,
            &serde_json::to_string(&create_call_member_event_content()).unwrap()
        );
    }

    #[test]
    fn deserialize_call_member_event_content() {
        fn create_call_member_event_content() -> CallMemberEventContent {
            CallMemberEventContent::new(vec![
                Membership {
                    application: Application::Call(CallApplicationContent {
                        call_id: "123456".to_owned(),
                        scope: CallScope::Room,
                    }),
                    device_id: "THIS_DEVICE".to_owned(),
                    expires: 36000,
                    foci_active: vec![Foci {
                        livekit_alias: "room1".to_owned(),
                        livekit_service_url: "https://livekit1.com".to_owned(), /* Url::parse("https://livekit.com"), */
                        foci_type: "livekit".to_owned(),
                    }],
                    membership_id: "0".to_owned(),
                },
                Membership {
                    application: Application::Call(CallApplicationContent {
                        call_id: "".to_owned(),
                        scope: CallScope::Room,
                    }),
                    device_id: "OTHER_DEVICE".to_owned(),
                    expires: 36000,
                    foci_active: vec![Foci {
                        livekit_alias: "room2".to_owned(),
                        livekit_service_url: "https://livekit2.com".to_owned(), /* Url::parse("https://livekit.com"), */
                        foci_type: "livekit".to_owned(),
                    }],
                    membership_id: "0".to_owned(),
                },
            ])
        }

        let call_member_event: &str = &_remove_whitespace(
            r#"
            {
                "memberships": [
                    {
                        "application": "m.call",
                        "call_id": "123456",
                        "scope": "m.room",
                        "device_id": "THIS_DEVICE",
                        "expires": 36000,
                        "foci_active": [
                            {
                                "livekit_alias": "room1",
                                "livekit_service_url": "https://livekit1.com",
                                "type": "livekit"
                            }
                        ],
                        "membershipID": "0"
                    },
                    {
                        "application": "m.call",
                        "call_id": "",
                        "scope": "m.room",
                        "device_id": "OTHER_DEVICE",
                        "expires": 36000,
                        "foci_active": [
                            {
                                "livekit_alias": "room2",
                                "livekit_service_url": "https://livekit2.com",
                                "type": "livekit"
                            }
                        ],
                        "membershipID": "0"
                    }
                ]
            }
            "#,
        );
        let ev_content: CallMemberEventContent = serde_json::from_str(call_member_event).unwrap();
        assert_eq!(ev_content, create_call_member_event_content());
    }
}
