//! Types to deserialize `m.room.join_rules` events.

use std::ops::Deref;

use ruma_common::serde::{from_raw_json_value, PartialEqAsRefStr, StringEnum};
use serde::Deserialize;

use super::Event;

/// A helper type for an [`Event`] of type `m.room.join_rules`.
///
/// This is a type that deserializes each field lazily, when requested.
#[derive(Debug, Clone)]
pub struct RoomJoinRulesEvent<E: Event>(E);

impl<E: Event> RoomJoinRulesEvent<E> {
    /// Construct a new `RoomJoinRulesEvent` around the given event.
    pub fn new(event: E) -> Self {
        Self(event)
    }

    /// The join rule of the room.
    pub fn join_rule(&self) -> Result<JoinRule, String> {
        #[derive(Deserialize)]
        struct RoomJoinRulesContentJoinRule {
            join_rule: JoinRule,
        }

        let content: RoomJoinRulesContentJoinRule =
            from_raw_json_value(self.content()).map_err(|err: serde_json::Error| {
                format!("missing or invalid `join_rule` field in `m.room.join_rules` event: {err}")
            })?;
        Ok(content.join_rule)
    }
}

impl<E: Event> Deref for RoomJoinRulesEvent<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, StringEnum, PartialEqAsRefStr)]
#[ruma_enum(rename_all = "snake_case")]
#[non_exhaustive]
pub enum JoinRule {
    /// `public`
    Public,

    /// `invite`
    Invite,

    /// `knock`
    Knock,

    /// `restricted`
    Restricted,

    /// `KnockRestricted`
    KnockRestricted,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

impl Eq for JoinRule {}

// Wrapper around `Box<str>` that cannot be used in a meaningful way outside of
// this crate. Used for string enums because their `_Custom` variant can't be
// truly private (only `#[doc(hidden)]`).
#[derive(Debug, Clone)]
pub struct PrivOwnedStr(Box<str>);
