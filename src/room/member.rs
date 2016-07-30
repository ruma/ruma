//! Types for the *m.room.member* event.

use std::fmt::{Display, Formatter, Error as FmtError};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use {StateEvent, ParseError, Visitor};
use stripped::StrippedState;

/// The current membership state of a user in the room.
///
/// Adjusts the membership state for a user in a room. It is preferable to use the membership APIs
/// (``/rooms/<room id>/invite`` etc) when performing membership actions rather than adjusting the
/// state directly as there are a restricted set of valid transformations. For example, user A
/// cannot force user B to join a room, and trying to force this state change directly will fail.
///
/// The *third_party_invite* property will be set if this invite is an *invite* event and is the
/// successor of an *m.room.third_party_invite* event, and absent otherwise.
///
/// This event may also include an *invite_room_state* key outside the *content* key. If present,
/// this contains an array of `StrippedState` events. These events provide information on a few
/// select state events such as the room name.
pub type MemberEvent = StateEvent<MemberEventContent, MemberEventExtraContent>;

/// The payload of a `MemberEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct MemberEventContent {
    /// The avatar URL for this user.
    pub avatar_url: Option<String>,

    /// The display name for this user.
    pub displayname: Option<String>,

    /// The membership state of this user.
    pub membership: MembershipState,

    /// Warning: This field is not implemented yet and its type will change!
    pub third_party_invite: (), // TODO
}

/// The membership state of a user.
#[derive(Debug, PartialEq)]
pub enum MembershipState {
    /// The user is banned.
    Ban,

    /// The user has been invited.
    Invite,

    /// The user has joined.
    Join,

    /// The user has requested to join.
    Knock,

    /// The user has left.
    Leave,
}

/// Extra content for a `MemberEvent`.
#[derive(Debug, Deserialize, Serialize)]
pub struct MemberEventExtraContent {
    /// A subset of the state of the room at the time of the invite.
    pub invite_room_state: Option<Vec<StrippedState>>,
}

impl Display for MembershipState {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let membership_state_str = match *self {
            MembershipState::Ban => "ban",
            MembershipState::Invite => "invite",
            MembershipState::Join => "join",
            MembershipState::Knock => "knock",
            MembershipState::Leave => "leave",
        };

        write!(f, "{}", membership_state_str)
    }
}

impl FromStr for MembershipState {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ban" => Ok(MembershipState::Ban),
            "invite" => Ok(MembershipState::Invite),
            "join" => Ok(MembershipState::Join),
            "knock" => Ok(MembershipState::Knock),
            "leave" => Ok(MembershipState::Leave),
            _ => Err(ParseError),
        }
    }
}

impl Serialize for MembershipState {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Deserialize for MembershipState {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> where D: Deserializer {
        deserializer.deserialize_str(Visitor::new())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_string};

    use super::MembershipState;

    #[test]
    fn membership_states_serialize_to_display_form() {
        assert_eq!(
            to_string(&MembershipState::Ban).unwrap(),
            r#""ban""#
        );
    }

    #[test]
    fn membership_states_deserialize_from_display_form() {
        assert_eq!(
            from_str::<MembershipState>(r#""ban""#).unwrap(),
            MembershipState::Ban
        );
    }

    #[test]
    fn invalid_membership_states_fail_deserialization() {
        assert!(from_str::<MembershipState>(r#""bad""#).is_err());
    }
}
