//! Modules for events in the *m.room* namespace.
//!
//! This module also contains types shared by events in its child namespaces.

pub mod aliases;
pub mod avatar;
pub mod canonical_alias;
pub mod create;
pub mod guest_access;
pub mod history_visibility;
pub mod join_rules;
pub mod member;
pub mod message;
pub mod name;
pub mod power_levels;
pub mod redaction;
pub mod third_party_invite;
pub mod topic;

/// Metadata about an image.
pub struct ImageInfo<'a> {
    height: u64,
    mimetype: &'a str,
    size: u64,
    width: u64,
}
