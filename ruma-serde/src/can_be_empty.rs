pub trait CanBeEmpty {
    /// Check whether `self` is empty.
    fn is_empty(&self) -> bool;
}

/// Check whether a value is empty.
pub fn is_empty<T: CanBeEmpty>(val: &T) -> bool {
    val.is_empty()
}
