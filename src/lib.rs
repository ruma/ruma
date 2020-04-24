//! De-/serialization helpers for other ruma crates

pub mod duration;
pub mod json_string;
pub mod time;
pub mod urlencoded;

pub fn is_default<T: Default + PartialEq>(val: &T) -> bool {
    val == &T::default()
}
