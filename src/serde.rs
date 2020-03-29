//! Modules to hold functions for de-/serializing remote types

pub mod duration;

pub fn is_default<T: Default + PartialEq>(val: &T) -> bool {
    val == &T::default()
}
