//! Constructors for [predefined push rules].
//!
//! [predefined push rules]: https://spec.matrix.org/latest/client-server-api/#predefined-rules

use ruma_macros::StringEnum;

use super::{
    Action::*, ConditionalPushRule, PatternedPushRule, PushCondition::*, RoomMemberCountIs,
    Ruleset, Tweak,
};
use crate::{PrivOwnedStr, UserId};

impl Ruleset {
    /// The list of all [predefined push rules].
    ///
    /// # Parameters
    ///
    /// - `user_id`: the user for which to generate the default rules. Some rules depend on the
    ///   user's ID (for instance those to send notifications when they are mentioned).
    ///
    /// [predefined push rules]: https://spec.matrix.org/latest/client-server-api/#predefined-rules
    pub fn server_default(user_id: &UserId) -> Self {
        Self {
            content: [
                #[allow(deprecated)]
                PatternedPushRule::contains_user_name(user_id),
            ]
            .into(),
            override_: [
                ConditionalPushRule::master(),
                ConditionalPushRule::suppress_notices(),
                ConditionalPushRule::invite_for_me(user_id),
                ConditionalPushRule::member_event(),
                ConditionalPushRule::is_user_mention(user_id),
                #[allow(deprecated)]
                ConditionalPushRule::contains_display_name(),
                ConditionalPushRule::is_room_mention(),
                #[allow(deprecated)]
                ConditionalPushRule::roomnotif(),
                ConditionalPushRule::tombstone(),
                ConditionalPushRule::reaction(),
                ConditionalPushRule::server_acl(),
                #[cfg(feature = "unstable-msc3958")]
                ConditionalPushRule::suppress_edits(),
                #[cfg(feature = "unstable-msc3930")]
                ConditionalPushRule::poll_response(),
            ]
            .into(),
            underride: [
                ConditionalPushRule::call(),
                ConditionalPushRule::encrypted_room_one_to_one(),
                ConditionalPushRule::room_one_to_one(),
                ConditionalPushRule::message(),
                ConditionalPushRule::encrypted(),
                #[cfg(feature = "unstable-msc3930")]
                ConditionalPushRule::poll_start_one_to_one(),
                #[cfg(feature = "unstable-msc3930")]
                ConditionalPushRule::poll_start(),
                #[cfg(feature = "unstable-msc3930")]
                ConditionalPushRule::poll_end_one_to_one(),
                #[cfg(feature = "unstable-msc3930")]
                ConditionalPushRule::poll_end(),
            ]
            .into(),
            ..Default::default()
        }
    }

    /// Update this ruleset with the given server-default push rules.
    ///
    /// This will replace the server-default rules in this ruleset (with `default` set to `true`)
    /// with the given ones while keeping the `enabled` and `actions` fields in the same state.
    ///
    /// The default rules in this ruleset that are not in the new server-default rules are removed.
    ///
    /// # Parameters
    ///
    /// - `server_default`: the new server-default push rules. This ruleset must not contain
    ///   non-default rules.
    pub fn update_with_server_default(&mut self, mut new_server_default: Ruleset) {
        // Copy the default rules states from the old rules to the new rules and remove the
        // server-default rules from the old rules.
        macro_rules! copy_rules_state {
            ($new_ruleset:ident, $old_ruleset:ident, @fields $($field_name:ident),+) => {
                $(
                    $new_ruleset.$field_name = $new_ruleset
                        .$field_name
                        .into_iter()
                        .map(|mut new_rule| {
                            if let Some(old_rule) =
                                $old_ruleset.$field_name.take(new_rule.rule_id.as_str())
                            {
                                new_rule.enabled = old_rule.enabled;
                                new_rule.actions = old_rule.actions;
                            }

                            new_rule
                        })
                        .collect();
                )+
            };
        }
        copy_rules_state!(new_server_default, self, @fields override_, content, room, sender, underride);

        // Remove the remaining server-default rules from the old rules.
        macro_rules! remove_remaining_default_rules {
            ($ruleset:ident, @fields $($field_name:ident),+) => {
                $(
                    $ruleset.$field_name.retain(|rule| !rule.default);
                )+
            };
        }
        remove_remaining_default_rules!(self, @fields override_, content, room, sender, underride);

        // `.m.rule.master` comes before all other push rules, while the other server-default push
        // rules come after.
        if let Some(master_rule) =
            new_server_default.override_.take(PredefinedOverrideRuleId::Master.as_str())
        {
            let (pos, _) = self.override_.insert_full(master_rule);
            self.override_.move_index(pos, 0);
        }

        // Merge the new server-default rules into the old rules.
        macro_rules! merge_rules {
            ($old_ruleset:ident, $new_ruleset:ident, @fields $($field_name:ident),+) => {
                $(
                    $old_ruleset.$field_name.extend($new_ruleset.$field_name);
                )+
            };
        }
        merge_rules!(self, new_server_default, @fields override_, content, room, sender, underride);
    }
}

/// Default override push rules
impl ConditionalPushRule {
    /// Matches all events, this can be enabled to turn off all push notifications other than those
    /// generated by override rules set by the user.
    pub fn master() -> Self {
        Self {
            actions: vec![],
            default: true,
            enabled: false,
            rule_id: PredefinedOverrideRuleId::Master.to_string(),
            conditions: vec![],
        }
    }

    /// Matches messages with a `msgtype` of `notice`.
    pub fn suppress_notices() -> Self {
        Self {
            actions: vec![],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::SuppressNotices.to_string(),
            conditions: vec![EventMatch {
                key: "content.msgtype".into(),
                pattern: "m.notice".into(),
            }],
        }
    }

    /// Matches any invites to a new room for this user.
    pub fn invite_for_me(user_id: &UserId) -> Self {
        Self {
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(false)),
            ],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::InviteForMe.to_string(),
            conditions: vec![
                EventMatch { key: "type".into(), pattern: "m.room.member".into() },
                EventMatch { key: "content.membership".into(), pattern: "invite".into() },
                EventMatch { key: "state_key".into(), pattern: user_id.to_string() },
            ],
        }
    }

    /// Matches any `m.room.member_event`.
    pub fn member_event() -> Self {
        Self {
            actions: vec![],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::MemberEvent.to_string(),
            conditions: vec![EventMatch { key: "type".into(), pattern: "m.room.member".into() }],
        }
    }

    /// Matches any message which contains the user’s Matrix ID in the list of `user_ids` under the
    /// `m.mentions` property.
    pub fn is_user_mention(user_id: &UserId) -> Self {
        Self {
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".to_owned())),
                SetTweak(Tweak::Highlight(true)),
            ],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::IsUserMention.to_string(),
            conditions: vec![EventPropertyContains {
                key: r"content.m\.mentions.user_ids".to_owned(),
                value: user_id.as_str().into(),
            }],
        }
    }

    /// Matches any message whose content is unencrypted and contains the user's current display
    /// name in the room in which it was sent.
    ///
    /// Since Matrix 1.7, this rule only matches if the event's content does not contain an
    /// `m.mentions` property.
    #[deprecated = "Since Matrix 1.7. Use the m.mentions property with ConditionalPushRule::is_user_mention() instead."]
    pub fn contains_display_name() -> Self {
        #[allow(deprecated)]
        Self {
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(true)),
            ],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::ContainsDisplayName.to_string(),
            conditions: vec![ContainsDisplayName],
        }
    }

    /// Matches any state event whose type is `m.room.tombstone`. This
    /// is intended to notify users of a room when it is upgraded,
    /// similar to what an `@room` notification would accomplish.
    pub fn tombstone() -> Self {
        Self {
            actions: vec![Notify, SetTweak(Tweak::Highlight(true))],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::Tombstone.to_string(),
            conditions: vec![
                EventMatch { key: "type".into(), pattern: "m.room.tombstone".into() },
                EventMatch { key: "state_key".into(), pattern: "".into() },
            ],
        }
    }

    /// Matches any message from a sender with the proper power level with the `room` property of
    /// the `m.mentions` property set to `true`.
    pub fn is_room_mention() -> Self {
        Self {
            actions: vec![Notify, SetTweak(Tweak::Highlight(true))],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::IsRoomMention.to_string(),
            conditions: vec![
                EventPropertyIs { key: r"content.m\.mentions.room".to_owned(), value: true.into() },
                SenderNotificationPermission { key: "room".to_owned() },
            ],
        }
    }

    /// Matches any message whose content is unencrypted and contains the text `@room`, signifying
    /// the whole room should be notified of the event.
    ///
    /// Since Matrix 1.7, this rule only matches if the event's content does not contain an
    /// `m.mentions` property.
    #[deprecated = "Since Matrix 1.7. Use the m.mentions property with ConditionalPushRule::is_room_mention() instead."]
    pub fn roomnotif() -> Self {
        #[allow(deprecated)]
        Self {
            actions: vec![Notify, SetTweak(Tweak::Highlight(true))],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::RoomNotif.to_string(),
            conditions: vec![
                EventMatch { key: "content.body".into(), pattern: "@room".into() },
                SenderNotificationPermission { key: "room".into() },
            ],
        }
    }

    /// Matches [reactions] to a message.
    ///
    /// [reactions]: https://spec.matrix.org/latest/client-server-api/#event-annotations-and-reactions
    pub fn reaction() -> Self {
        Self {
            actions: vec![],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::Reaction.to_string(),
            conditions: vec![EventMatch { key: "type".into(), pattern: "m.reaction".into() }],
        }
    }

    /// Matches [room server ACLs].
    ///
    /// [room server ACLs]: https://spec.matrix.org/latest/client-server-api/#server-access-control-lists-acls-for-rooms
    pub fn server_acl() -> Self {
        Self {
            actions: vec![],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::RoomServerAcl.to_string(),
            conditions: vec![
                EventMatch { key: "type".into(), pattern: "m.room.server_acl".into() },
                EventMatch { key: "state_key".into(), pattern: "".into() },
            ],
        }
    }

    /// Matches [event replacements].
    ///
    /// [event replacements]: https://spec.matrix.org/latest/client-server-api/#event-replacements
    #[cfg(feature = "unstable-msc3958")]
    pub fn suppress_edits() -> Self {
        Self {
            actions: vec![],
            default: true,
            enabled: true,
            rule_id: PredefinedOverrideRuleId::SuppressEdits.to_string(),
            conditions: vec![EventPropertyIs {
                key: r"content.m\.relates_to.rel_type".to_owned(),
                value: "m.replace".into(),
            }],
        }
    }

    /// Matches a poll response event sent in any room.
    ///
    /// This rule uses the unstable prefixes defined in [MSC3381] and [MSC3930].
    ///
    /// [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    pub fn poll_response() -> Self {
        Self {
            rule_id: PredefinedOverrideRuleId::PollResponse.to_string(),
            default: true,
            enabled: true,
            conditions: vec![EventPropertyIs {
                key: "type".to_owned(),
                value: "org.matrix.msc3381.poll.response".into(),
            }],
            actions: vec![],
        }
    }
}

/// Default content push rules
impl PatternedPushRule {
    /// Matches any message whose content is unencrypted and contains the local part of the user's
    /// Matrix ID, separated by word boundaries.
    ///
    /// Since Matrix 1.7, this rule only matches if the event's content does not contain an
    /// `m.mentions` property.
    #[deprecated = "Since Matrix 1.7. Use the m.mentions property with ConditionalPushRule::is_user_mention() instead."]
    pub fn contains_user_name(user_id: &UserId) -> Self {
        #[allow(deprecated)]
        Self {
            rule_id: PredefinedContentRuleId::ContainsUserName.to_string(),
            enabled: true,
            default: true,
            pattern: user_id.localpart().into(),
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(true)),
            ],
        }
    }
}

/// Default underrides push rules
impl ConditionalPushRule {
    /// Matches any incoming VOIP call.
    pub fn call() -> Self {
        Self {
            rule_id: PredefinedUnderrideRuleId::Call.to_string(),
            default: true,
            enabled: true,
            conditions: vec![EventMatch { key: "type".into(), pattern: "m.call.invite".into() }],
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("ring".into())),
                SetTweak(Tweak::Highlight(false)),
            ],
        }
    }

    /// Matches any encrypted event sent in a room with exactly two members.
    ///
    /// Unlike other push rules, this rule cannot be matched against the content of the event by
    /// nature of it being encrypted. This causes the rule to be an "all or nothing" match where it
    /// either matches all events that are encrypted (in 1:1 rooms) or none.
    pub fn encrypted_room_one_to_one() -> Self {
        Self {
            rule_id: PredefinedUnderrideRuleId::EncryptedRoomOneToOne.to_string(),
            default: true,
            enabled: true,
            conditions: vec![
                RoomMemberCount { is: RoomMemberCountIs::from(js_int::uint!(2)) },
                EventMatch { key: "type".into(), pattern: "m.room.encrypted".into() },
            ],
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(false)),
            ],
        }
    }

    /// Matches any message sent in a room with exactly two members.
    pub fn room_one_to_one() -> Self {
        Self {
            rule_id: PredefinedUnderrideRuleId::RoomOneToOne.to_string(),
            default: true,
            enabled: true,
            conditions: vec![
                RoomMemberCount { is: RoomMemberCountIs::from(js_int::uint!(2)) },
                EventMatch { key: "type".into(), pattern: "m.room.message".into() },
            ],
            actions: vec![
                Notify,
                SetTweak(Tweak::Sound("default".into())),
                SetTweak(Tweak::Highlight(false)),
            ],
        }
    }

    /// Matches all chat messages.
    pub fn message() -> Self {
        Self {
            rule_id: PredefinedUnderrideRuleId::Message.to_string(),
            default: true,
            enabled: true,
            conditions: vec![EventMatch { key: "type".into(), pattern: "m.room.message".into() }],
            actions: vec![Notify, SetTweak(Tweak::Highlight(false))],
        }
    }

    /// Matches all encrypted events.
    ///
    /// Unlike other push rules, this rule cannot be matched against the content of the event by
    /// nature of it being encrypted. This causes the rule to be an "all or nothing" match where it
    /// either matches all events that are encrypted (in group rooms) or none.
    pub fn encrypted() -> Self {
        Self {
            rule_id: PredefinedUnderrideRuleId::Encrypted.to_string(),
            default: true,
            enabled: true,
            conditions: vec![EventMatch { key: "type".into(), pattern: "m.room.encrypted".into() }],
            actions: vec![Notify, SetTweak(Tweak::Highlight(false))],
        }
    }

    /// Matches a poll start event sent in a room with exactly two members.
    ///
    /// This rule uses the unstable prefixes defined in [MSC3381] and [MSC3930].
    ///
    /// [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    pub fn poll_start_one_to_one() -> Self {
        Self {
            rule_id: PredefinedUnderrideRuleId::PollStartOneToOne.to_string(),
            default: true,
            enabled: true,
            conditions: vec![
                RoomMemberCount { is: RoomMemberCountIs::from(js_int::uint!(2)) },
                EventPropertyIs {
                    key: "type".to_owned(),
                    value: "org.matrix.msc3381.poll.start".into(),
                },
            ],
            actions: vec![Notify, SetTweak(Tweak::Sound("default".into()))],
        }
    }

    /// Matches a poll start event sent in any room.
    ///
    /// This rule uses the unstable prefixes defined in [MSC3381] and [MSC3930].
    ///
    /// [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    pub fn poll_start() -> Self {
        Self {
            rule_id: PredefinedUnderrideRuleId::PollStart.to_string(),
            default: true,
            enabled: true,
            conditions: vec![EventPropertyIs {
                key: "type".to_owned(),
                value: "org.matrix.msc3381.poll.start".into(),
            }],
            actions: vec![Notify],
        }
    }

    /// Matches a poll end event sent in a room with exactly two members.
    ///
    /// This rule uses the unstable prefixes defined in [MSC3381] and [MSC3930].
    ///
    /// [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    pub fn poll_end_one_to_one() -> Self {
        Self {
            rule_id: PredefinedUnderrideRuleId::PollEndOneToOne.to_string(),
            default: true,
            enabled: true,
            conditions: vec![
                RoomMemberCount { is: RoomMemberCountIs::from(js_int::uint!(2)) },
                EventPropertyIs {
                    key: "type".to_owned(),
                    value: "org.matrix.msc3381.poll.end".into(),
                },
            ],
            actions: vec![Notify, SetTweak(Tweak::Sound("default".into()))],
        }
    }

    /// Matches a poll end event sent in any room.
    ///
    /// This rule uses the unstable prefixes defined in [MSC3381] and [MSC3930].
    ///
    /// [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    pub fn poll_end() -> Self {
        Self {
            rule_id: PredefinedUnderrideRuleId::PollEnd.to_string(),
            default: true,
            enabled: true,
            conditions: vec![EventPropertyIs {
                key: "type".to_owned(),
                value: "org.matrix.msc3381.poll.end".into(),
            }],
            actions: vec![Notify],
        }
    }
}

/// The rule IDs of the predefined server push rules.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PredefinedRuleId {
    /// User-configured rules that override all other kinds.
    Override(PredefinedOverrideRuleId),

    /// Lowest priority user-defined rules.
    Underride(PredefinedUnderrideRuleId),

    /// Content-specific rules.
    Content(PredefinedContentRuleId),
}

/// The rule IDs of the predefined override server push rules.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[ruma_enum(rename_all = ".m.rule.snake_case")]
#[non_exhaustive]
pub enum PredefinedOverrideRuleId {
    /// `.m.rule.master`
    Master,

    /// `.m.rule.suppress_notices`
    SuppressNotices,

    /// `.m.rule.invite_for_me`
    InviteForMe,

    /// `.m.rule.member_event`
    MemberEvent,

    /// `.m.rule.is_user_mention`
    IsUserMention,

    /// `.m.rule.contains_display_name`
    #[deprecated = "Since Matrix 1.7. Use the m.mentions property with PredefinedOverrideRuleId::IsUserMention instead."]
    ContainsDisplayName,

    /// `.m.rule.is_room_mention`
    IsRoomMention,

    /// `.m.rule.roomnotif`
    #[ruma_enum(rename = ".m.rule.roomnotif")]
    #[deprecated = "Since Matrix 1.7. Use the m.mentions property with PredefinedOverrideRuleId::IsRoomMention instead."]
    RoomNotif,

    /// `.m.rule.tombstone`
    Tombstone,

    /// `.m.rule.reaction`
    Reaction,

    /// `.m.rule.room.server_acl`
    #[ruma_enum(rename = ".m.rule.room.server_acl")]
    RoomServerAcl,

    /// `.m.rule.suppress_edits`
    #[cfg(feature = "unstable-msc3958")]
    #[ruma_enum(rename = ".org.matrix.msc3958.suppress_edits")]
    SuppressEdits,

    /// `.m.rule.poll_response`
    ///
    /// This uses the unstable prefix defined in [MSC3930].
    ///
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    #[ruma_enum(rename = ".org.matrix.msc3930.rule.poll_response")]
    PollResponse,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The rule IDs of the predefined underride server push rules.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[ruma_enum(rename_all = ".m.rule.snake_case")]
#[non_exhaustive]
pub enum PredefinedUnderrideRuleId {
    /// `.m.rule.call`
    Call,

    /// `.m.rule.encrypted_room_one_to_one`
    EncryptedRoomOneToOne,

    /// `.m.rule.room_one_to_one`
    RoomOneToOne,

    /// `.m.rule.message`
    Message,

    /// `.m.rule.encrypted`
    Encrypted,

    /// `.m.rule.poll_start_one_to_one`
    ///
    /// This uses the unstable prefix defined in [MSC3930].
    ///
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    #[ruma_enum(rename = ".org.matrix.msc3930.rule.poll_start_one_to_one")]
    PollStartOneToOne,

    /// `.m.rule.poll_start`
    ///
    /// This uses the unstable prefix defined in [MSC3930].
    ///
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    #[ruma_enum(rename = ".org.matrix.msc3930.rule.poll_start")]
    PollStart,

    /// `.m.rule.poll_end_one_to_one`
    ///
    /// This uses the unstable prefix defined in [MSC3930].
    ///
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    #[ruma_enum(rename = ".org.matrix.msc3930.rule.poll_end_one_to_one")]
    PollEndOneToOne,

    /// `.m.rule.poll_end`
    ///
    /// This uses the unstable prefix defined in [MSC3930].
    ///
    /// [MSC3930]: https://github.com/matrix-org/matrix-spec-proposals/pull/3930
    #[cfg(feature = "unstable-msc3930")]
    #[ruma_enum(rename = ".org.matrix.msc3930.rule.poll_end")]
    PollEnd,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The rule IDs of the predefined content server push rules.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, StringEnum)]
#[ruma_enum(rename_all = ".m.rule.snake_case")]
#[non_exhaustive]
pub enum PredefinedContentRuleId {
    /// `.m.rule.contains_user_name`
    #[deprecated = "Since Matrix 1.7. Use the m.mentions property with PredefinedOverrideRuleId::IsUserMention instead."]
    ContainsUserName,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use assign::assign;

    use super::PredefinedOverrideRuleId;
    use crate::{
        push::{Action, ConditionalPushRule, ConditionalPushRuleInit, Ruleset},
        user_id,
    };

    #[test]
    fn update_with_server_default() {
        let user_rule_id = "user_always_true";
        let default_rule_id = ".default_always_true";

        let override_ = [
            // Default `.m.rule.master` push rule with non-default state.
            assign!(ConditionalPushRule::master(), { enabled: true, actions: vec![Action::Notify]}),
            // User-defined push rule.
            ConditionalPushRuleInit {
                actions: vec![],
                default: false,
                enabled: false,
                rule_id: user_rule_id.to_owned(),
                conditions: vec![],
            }
            .into(),
            // Old server-default push rule.
            ConditionalPushRuleInit {
                actions: vec![],
                default: true,
                enabled: true,
                rule_id: default_rule_id.to_owned(),
                conditions: vec![],
            }
            .into(),
        ]
        .into_iter()
        .collect();
        let mut ruleset = Ruleset { override_, ..Default::default() };

        let new_server_default = Ruleset::server_default(user_id!("@user:localhost"));

        ruleset.update_with_server_default(new_server_default);

        // Master rule is in first position.
        let master_rule = &ruleset.override_[0];
        assert_eq!(master_rule.rule_id, PredefinedOverrideRuleId::Master.as_str());

        // `enabled` and `actions` have been copied from the old rules.
        assert!(master_rule.enabled);
        assert_eq!(master_rule.actions.len(), 1);
        assert_matches!(&master_rule.actions[0], Action::Notify);

        // Non-server-default rule is still present and hasn't changed.
        let user_rule = ruleset.override_.get(user_rule_id).unwrap();
        assert!(!user_rule.enabled);
        assert_eq!(user_rule.actions.len(), 0);

        // Old server-default rule is gone.
        assert_matches!(ruleset.override_.get(default_rule_id), None);

        // New server-default rule is present and hasn't changed.
        let member_event_rule =
            ruleset.override_.get(PredefinedOverrideRuleId::MemberEvent.as_str()).unwrap();
        assert!(member_event_rule.enabled);
        assert_eq!(member_event_rule.actions.len(), 0);
    }
}
