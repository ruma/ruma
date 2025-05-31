//! Types for the [`m.invite_permission_config`] account data.
//!
//! [`m.invite_permission_config`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155

use std::ops::Deref;

use ruma_common::UserId;
use serde::{Deserialize, Serialize};
use wildmatch::WildMatch;

use crate::macros::EventContent;

/// The content of an [`m.invite_permission_config`] account data.
///
/// All the lists contain [glob expressions] to match against the sender of the invites. For the
/// users lists the full user ID is matched, and for the servers lists only the server name is
/// matched.
///
/// The lists are checked in the following order:
///
/// 1. `allowed_users`
/// 2. `ignored_users`
/// 3. `blocked_users`
/// 4. `allowed_servers`
/// 5. `ignored_users`
/// 6. `blocked_users`
///
/// If a decision was not reached after checking all the lists, the invitation is allowed, which
/// means that if there are no ignored or blocked users or servers, all users are allowed to send an
/// invitation.
///
/// [`m.invite_permission_config`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "m.invite_permission_config", kind = GlobalAccountData)]
pub struct InvitePermissionConfigEventContent {
    /// The users whose invites are explicitly allowed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_users: Vec<String>,

    /// The users whose invites are ignored.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignored_users: Vec<String>,

    /// The users whose invites are blocked.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_users: Vec<String>,

    /// The servers from which invites are explicitly allowed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_servers: Vec<String>,

    /// The servers from which invites are ignored.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignored_servers: Vec<String>,

    /// The servers from which invites are blocked.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_servers: Vec<String>,
}

impl InvitePermissionConfigEventContent {
    /// Construct an empty `InvitePermissionConfigEventContent`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether all invites are allowed according to this account data.
    fn are_all_invites_allowed(&self) -> bool {
        self.ignored_users.is_empty()
            && self.blocked_users.is_empty()
            && self.ignored_servers.is_empty()
            && self.blocked_servers.is_empty()
    }

    /// Get the decision for the given invite sender according to this account data.
    pub fn decision_for_sender(&self, sender: &UserId) -> InvitePermissionDecision {
        if self.are_all_invites_allowed() {
            return InvitePermissionDecision::Allow;
        }

        fn list_has_match(list: &[String], string: &str) -> bool {
            list.iter().any(|pattern| WildMatch::new(pattern).matches(string))
        }

        let sender_str = sender.as_str();

        if list_has_match(&self.allowed_users, sender_str) {
            return InvitePermissionDecision::Allow;
        }

        if list_has_match(&self.ignored_users, sender_str) {
            return InvitePermissionDecision::Ignore;
        }

        if list_has_match(&self.blocked_users, sender_str) {
            return InvitePermissionDecision::Block;
        }

        let server_name = sender.server_name().as_str();

        if list_has_match(&self.allowed_servers, server_name) {
            return InvitePermissionDecision::Allow;
        }

        if list_has_match(&self.ignored_servers, server_name) {
            return InvitePermissionDecision::Ignore;
        }

        if list_has_match(&self.blocked_servers, server_name) {
            return InvitePermissionDecision::Block;
        }

        InvitePermissionDecision::Allow
    }
}

/// The unstable version of [`InvitePermissionConfigEventContent`], using the unstable prefix
/// defined in [MSC4155].
///
/// [MSC4155]: https://github.com/matrix-org/matrix-spec-proposals/pull/4155
#[derive(Clone, Debug, Default, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "org.matrix.msc4155.invite_permission_config", kind = GlobalAccountData)]
#[serde(transparent)]
pub struct UnstableInvitePermissionConfigEventContent(pub InvitePermissionConfigEventContent);

impl Deref for UnstableInvitePermissionConfigEventContent {
    type Target = InvitePermissionConfigEventContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<UnstableInvitePermissionConfigEventContent> for InvitePermissionConfigEventContent {
    fn from(value: UnstableInvitePermissionConfigEventContent) -> Self {
        value.0
    }
}

impl From<InvitePermissionConfigEventContent> for UnstableInvitePermissionConfigEventContent {
    fn from(value: InvitePermissionConfigEventContent) -> Self {
        Self(value)
    }
}

/// The decision for an invite according to an [`InvitePermissionConfigEventContent`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub enum InvitePermissionDecision {
    /// Allow the invite.
    #[default]
    Allow,

    /// Ignore the invite.
    ///
    /// The recipient should not see it and the sender should still see it as pending.
    Ignore,

    /// Block the invite.
    ///
    /// The recipient should not see it and it should be automatically refused or rejected.
    Block,
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_matches;
    use ruma_common::user_id;
    use serde_json::{from_value as from_json_value, json, to_value as to_json_value};

    use super::{InvitePermissionConfigEventContent, InvitePermissionDecision};
    use crate::AnyGlobalAccountDataEvent;

    #[test]
    fn unstable_serialization() {
        let json_account_data = json!({
            "type": "org.matrix.msc4155.invite_permission_config",
            "content": {
                "allowed_users": ["@lily:example.com"],
                "ignored_users": ["*"],
            },
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json_account_data.clone()).unwrap(),
            AnyGlobalAccountDataEvent::UnstableInvitePermissionConfig(account_data)
        );

        let content = &account_data.content;
        assert_eq!(content.allowed_users, ["@lily:example.com"]);
        assert_eq!(content.ignored_users, ["*"]);
        assert_eq!(content.blocked_users, &[] as &[String]);
        assert_eq!(content.allowed_servers, &[] as &[String]);
        assert_eq!(content.ignored_servers, &[] as &[String]);
        assert_eq!(content.blocked_servers, &[] as &[String]);

        assert_eq!(to_json_value(account_data).unwrap(), json_account_data);
    }

    #[test]
    fn stable_serialization() {
        let json_account_data = json!({
            "type": "m.invite_permission_config",
            "content": {
                "blocked_users": ["@lily:example.com"],
                "ignored_servers": ["*"],
            },
        });

        assert_matches!(
            from_json_value::<AnyGlobalAccountDataEvent>(json_account_data.clone()).unwrap(),
            AnyGlobalAccountDataEvent::InvitePermissionConfig(account_data)
        );

        let content = &account_data.content;
        assert_eq!(content.allowed_users, &[] as &[String]);
        assert_eq!(content.ignored_users, &[] as &[String]);
        assert_eq!(content.blocked_users, ["@lily:example.com"]);
        assert_eq!(content.allowed_servers, &[] as &[String]);
        assert_eq!(content.ignored_servers, ["*"]);
        assert_eq!(content.blocked_servers, &[] as &[String]);

        assert_eq!(to_json_value(account_data).unwrap(), json_account_data);
    }

    #[test]
    fn decision_for_sender() {
        // These tests are based on the examples from MSC4155.

        let good_guy = user_id!("@goodguy:goodguys.org");
        let good_guy_exception = user_id!("@notactuallyguy:goodguys.org");
        let bad_guy = user_id!("@badguy:badguys.org");
        let bad_guy_exception = user_id!("@goodguy:badguys.org");
        let really_bad_guy = user_id!("@anyone:reallybadguys.org");

        // Allow all invites.
        let json = json!({});
        let config = from_json_value::<InvitePermissionConfigEventContent>(json).unwrap();
        assert_eq!(config.decision_for_sender(good_guy), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(good_guy_exception), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(bad_guy), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(bad_guy_exception), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(really_bad_guy), InvitePermissionDecision::Allow);

        // Block all invites.
        let json = json!({ "blocked_servers": ["*"] });
        let config = from_json_value::<InvitePermissionConfigEventContent>(json).unwrap();
        assert_eq!(config.decision_for_sender(good_guy), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(good_guy_exception), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(bad_guy), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(bad_guy_exception), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(really_bad_guy), InvitePermissionDecision::Block);

        // Only allow invites from goodguys.org.
        let json = json!({
            "allowed_servers": ["goodguys.org"],
            "blocked_servers": ["*"],
        });
        let config = from_json_value::<InvitePermissionConfigEventContent>(json).unwrap();
        assert_eq!(config.decision_for_sender(good_guy), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(good_guy_exception), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(bad_guy), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(bad_guy_exception), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(really_bad_guy), InvitePermissionDecision::Block);

        // Block invites from badguys.org.
        let json = json!({
            "blocked_servers": ["badguys.org"],
        });
        let config = from_json_value::<InvitePermissionConfigEventContent>(json).unwrap();
        assert_eq!(config.decision_for_sender(good_guy), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(good_guy_exception), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(bad_guy), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(bad_guy_exception), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(really_bad_guy), InvitePermissionDecision::Allow);

        // Only allow invites from goodguys.org except for @notactuallyguy:goodguys.org.
        let json = json!({
            "blocked_users": ["@notactuallyguy:goodguys.org"],
            "allowed_servers": ["goodguys.org"],
            "blocked_servers": ["*"],
        });
        let config = from_json_value::<InvitePermissionConfigEventContent>(json).unwrap();
        assert_eq!(config.decision_for_sender(good_guy), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(good_guy_exception), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(bad_guy), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(bad_guy_exception), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(really_bad_guy), InvitePermissionDecision::Block);

        // Block invites from badguys.org except for @goodguy:badguys.org.
        let json = json!({
            "allowed_users": ["@goodguy:badguys.org"],
            "blocked_servers": ["badguys.org"],
        });
        let config = from_json_value::<InvitePermissionConfigEventContent>(json).unwrap();
        assert_eq!(config.decision_for_sender(good_guy), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(good_guy_exception), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(bad_guy), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(bad_guy_exception), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(really_bad_guy), InvitePermissionDecision::Allow);

        // Only allow invites from goodguys.org and ignore invites from reallybadguys.org.
        let json = json!({
            "allowed_servers": ["goodguys.org"],
            "ignored_servers": ["reallybadguys.org"],
            "blocked_servers": ["*"],
        });
        let config = from_json_value::<InvitePermissionConfigEventContent>(json).unwrap();
        assert_eq!(config.decision_for_sender(good_guy), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(good_guy_exception), InvitePermissionDecision::Allow);
        assert_eq!(config.decision_for_sender(bad_guy), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(bad_guy_exception), InvitePermissionDecision::Block);
        assert_eq!(config.decision_for_sender(really_bad_guy), InvitePermissionDecision::Ignore);
    }
}
