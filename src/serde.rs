//! Modules to hold functions for de-/serializing remote types

pub mod duration;
pub mod json_string;

pub fn is_default<T: Default + PartialEq>(val: &T) -> bool {
    val == &T::default()
}
