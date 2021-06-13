use std::{error::Error, fmt};

/// An error returned when attempting to create an event with data that would make it invalid.
///
/// This type is similar to [`InvalidEvent`](struct.InvalidEvent.html), but used during the
/// construction of a new event, as opposed to deserialization of an existing event from JSON.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidInput(pub(crate) String);

impl fmt::Display for InvalidInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for InvalidInput {}

/// An error when attempting to create a value from a string via the `FromStr` trait.
#[derive(Clone, Eq, Debug, Hash, PartialEq)]
#[cfg_attr(not(feature = "unstable-exhaustive-types"), non_exhaustive)]
pub struct FromStrError;

impl fmt::Display for FromStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse type from string")
    }
}

impl Error for FromStrError {}
