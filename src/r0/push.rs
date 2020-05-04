//! Endpoints for push notifications.

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

pub mod delete_pushrule;
pub mod get_notifications;
pub mod get_pushers;
pub mod get_pushrule;
pub mod get_pushrule_actions;
pub mod get_pushrule_enabled;
pub mod get_pushrules_all;
pub mod get_pushrules_global_scope;
pub mod set_pusher;
pub mod set_pushrule;
pub mod set_pushrule_actions;
pub mod set_pushrule_enabled;

pub use ruma_common::push::Action;

/// The kinds of push rules that are available
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display, EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum RuleKind {
    /// User-configured rules that override all other kinds
    Override,

    /// Lowest priority user-defined rules
    Underride,

    /// Sender-specific rules
    Sender,

    /// Room-specific rules
    Room,

    /// Content-specific rules
    Content,
}

impl TryFrom<&'_ str> for RuleKind {
    type Error = strum::ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// A push rule
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PushRule {
    /// The actions to perform when this rule is matched.
    pub actions: Vec<Action>,

    /// Whether this is a default rule, or has been set explicitly.
    pub default: bool,

    /// Whether the push rule is enabled or not.
    pub enabled: bool,

    /// The ID of this rule.
    pub rule_id: String,

    /// The conditions that must hold true for an event in order for a rule to be applied to an event. A rule with no conditions always matches.
    /// Only applicable to underride and override rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<PushCondition>>,

    /// The glob-style pattern to match against. Only applicable to content rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// A condition for a push rule
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")] // Using internally tagged enum representation to match the spec
pub enum PushCondition {
    /// This is a glob pattern match on a field of the event.
    EventMatch {
        /// The dot-separated field of the event to match, e.g. `content.body`
        key: String,

        /// The glob-style pattern to match against.
        pattern: String,
    },

    /// This matches unencrypted messages where `content.body` contains
    /// the owner's display name in that room.
    ContainsDisplayName,

    /// This matches the current number of members in the room.
    RoomMemberCount {
        /// A decimal integer optionally prefixed by one of, ==, <, >, >= or <=.
        /// Default prefix is ==.
        is: String,
    },

    /// This takes into account the current power levels in the room, ensuring the
    /// sender of the event has high enough power to trigger the notification.
    SenderNotificationPermission {
        /// A string that determines the power level the sender must have to
        /// trigger notifications of a given type, such as `room`.
        key: String,
    },
}

/// Defines a pusher
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pusher {
    /// This is a unique identifier for this pusher. Max length, 512 bytes.
    pub pushkey: String,

    /// The kind of the pusher. If set to None in a call to set_pusher, this
    /// will delete the pusher
    pub kind: Option<PusherKind>,

    /// This is a reverse-DNS style identifier for the application. Max length, 64 chars.
    pub app_id: String,

    /// A string that will allow the user to identify what application owns this pusher.
    pub app_display_name: String,

    /// A string that will allow the user to identify what device owns this pusher.
    pub device_display_name: String,

    /// This string determines which set of device specific rules this pusher executes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_tag: Option<String>,

    /// The preferred language for receiving notifications (e.g. 'en' or 'en-US')
    pub lang: String,

    /// Information for the pusher implementation itself.
    pub data: PusherData,
}

/// Which kind a pusher is
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PusherKind {
    /// A pusher that sends HTTP pokes.
    Http,

    /// A pusher that emails the user with unread notifications.
    Email,
}

/// Information for the pusher implementation itself.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PusherData {
    /// Required if the pusher's kind is http. The URL to use to send notifications to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// The format to use when sending notifications to the Push Gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<PushFormat>,
}

/// A special format that the homeserver should use when sending notifications to a Push Gateway.
/// Currently, only "event_id_only" is supported as of [Push Gateway API r0.1.1](https://matrix.org/docs/spec/push_gateway/r0.1.1#homeserver-behaviour)
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushFormat {
    /// Require the homeserver to only send a reduced set of fields in the push.
    EventIdOnly,
}
