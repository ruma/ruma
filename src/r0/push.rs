//! Endpoints for push notifications.

use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::{
    de::{Error as SerdeError, MapAccess, Unexpected, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value as JsonValue;

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
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
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

impl Display for RuleKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            RuleKind::Override => "override",
            RuleKind::Underride => "underride",
            RuleKind::Sender => "sender",
            RuleKind::Room => "room",
            RuleKind::Content => "content",
        };
        write!(f, "{}", s)
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

/// This represents the different actions that should be taken when a rule is matched, and
/// controls how notifications are delivered to the client.
// See https://matrix.org/docs/spec/client_server/r0.6.0#actions for details.
#[derive(Clone, Debug)]
pub enum Action {
    /// Causes matching events to generate a notification.
    Notify,

    /// Prevents matching events from generating a notification.
    DontNotify,

    /// Behaves like notify but homeservers may choose to coalesce multiple events
    /// into a single notification.
    Coalesce,

    /// Sets an entry in the 'tweaks' dictionary sent to the push gateway.
    SetTweak {
        /// The kind of this tweak
        kind: TweakKind,

        /// The value of the tweak, if any
        value: Option<JsonValue>,
    },
}

/// The different kinds of tweaks available
#[derive(Clone, Debug)]
pub enum TweakKind {
    /// The "sound" tweak.
    Sound,

    /// The "highlight" tweak.
    Highlight,

    /// A name for a custom client-defined tweak.
    Custom(String),
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Action::Notify => serializer.serialize_unit_variant("Action", 0, "notify"),
            Action::DontNotify => serializer.serialize_unit_variant("Action", 1, "dont_notify"),
            Action::Coalesce => serializer.serialize_unit_variant("Action", 2, "coalesce"),
            Action::SetTweak { kind, value } => {
                let kind_name = match &kind {
                    TweakKind::Sound => "sound",
                    TweakKind::Highlight => "highlight",
                    TweakKind::Custom(name) => name,
                };
                let num_fields = match value {
                    Some(_) => 2,
                    None => 1,
                };
                let mut s = serializer.serialize_struct("Action", num_fields)?;
                s.serialize_field("set_tweak", kind_name)?;

                match &value {
                    Some(value) => {
                        s.serialize_field("value", value)?;
                    }
                    None => {}
                };
                s.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ActionVisitor;
        impl<'de> Visitor<'de> for ActionVisitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
                write!(formatter, "a valid action object")
            }

            /// Match a simple action type
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                match v {
                    "notify" => Ok(Action::Notify),
                    "dont_notify" => Ok(Action::DontNotify),
                    "coalesce" => Ok(Action::Coalesce),
                    s => Err(E::unknown_variant(
                        &s,
                        &["notify", "dont_notify", "coalesce"],
                    )),
                }
            }

            /// Match the more complex set_tweaks action object as a key-value map
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut tweak_kind: Option<TweakKind> = None;
                let mut tweak_value: Option<JsonValue> = None;

                // We loop over all entries in the map to find one with a "set_tweak" key to find
                // which type of tweak is being set.
                // Then we also try to find one with the "value" key if it exists.
                while let Some((key, value)) = map.next_entry::<&str, JsonValue>()? {
                    match key {
                        "set_tweak" => {
                            let kind = match value.as_str() {
                                Some("sound") => TweakKind::Sound,
                                Some("highlight") => TweakKind::Highlight,
                                Some(s) => TweakKind::Custom(s.to_string()),
                                None => {
                                    return Err(A::Error::invalid_type(
                                        Unexpected::Other("non-string object"),
                                        &"string",
                                    ))
                                }
                            };
                            tweak_kind = Some(kind);
                        }
                        "value" => {
                            tweak_value = Some(value);
                        }
                        _ => {}
                    }
                }

                match tweak_kind {
                    Some(kind) => Ok(Action::SetTweak {
                        kind,
                        value: tweak_value,
                    }),
                    None => Err(A::Error::invalid_type(
                        Unexpected::Other("object without \"set_tweak\" key"),
                        &"valid \"set_tweak\" action object",
                    )),
                }
            }
        }

        deserializer.deserialize_any(ActionVisitor)
    }
}
