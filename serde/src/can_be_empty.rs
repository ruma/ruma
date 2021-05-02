//! Helpers for emptiness checks in `#[serde(skip_serializing_if)]`.

/// Trait for types that have an "empty" state.
///
/// If `Default` is implemented for `Self`, `Self::default().is_empty()` should always be `true`.
pub trait CanBeEmpty {
    /// Check whether `self` is empty.
    fn is_empty(&self) -> bool;
}

/// Check whether a value is empty.
pub fn is_empty<T: CanBeEmpty>(val: &T) -> bool {
    val.is_empty()
}
