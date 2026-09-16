use ruma_common::EventId;
use thiserror::Error;

/// Result type for state resolution.
pub type Result<T> = std::result::Result<T, Error>;

/// Represents the various errors that arise when resolving state.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The given event was not found.
    #[error("Failed to find event {0}")]
    NotFound(EventId),

    /// An auth event is invalid.
    #[error("Invalid auth event: {0}")]
    AuthEvent(String),

    /// A state event doesn't have a `state_key`.
    #[error("State event has no `state_key`")]
    MissingStateKey,

    /// Provided `fetch_conflicted_state_subgraph` function failed.
    #[error("`fetch_conflicted_state_subgraph` failed")]
    FetchConflictedStateSubgraphFailed,
}
