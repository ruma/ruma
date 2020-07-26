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
