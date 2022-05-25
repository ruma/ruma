//! Modules for events in the `m.poll` namespace ([MSC3381]).
//!
//! This module also contains types shared by events in its child namespaces.
//!
//! [MSC3381]: https://github.com/matrix-org/matrix-spec-proposals/pull/3381

use serde::{Deserialize, Serialize};

use crate::OwnedEventId;

pub mod end;
pub mod response;
pub mod start;

/// An `m.reference` relation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
#[serde(tag = "rel_type", rename = "m.reference")]
pub struct ReferenceRelation {
    /// The ID of the event this references.
    pub event_id: OwnedEventId,
}

impl ReferenceRelation {
    /// Creates a new `ReferenceRelation` that references the given event ID.
    pub fn new(event_id: OwnedEventId) -> Self {
        Self { event_id }
    }
}
