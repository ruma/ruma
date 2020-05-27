use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

/// An event that is malformed or otherwise invalid.
///
/// When attempting to deserialize an [`EventJson`](enum.EventJson.html), an error in the input
/// data may cause deserialization to fail, or the JSON structure may be correct, but additional
/// constraints defined in the matrix specification are not upheld. This type provides an error
/// message and a flag for which type of error was encountered.
#[derive(Clone, Debug)]
pub struct InvalidEvent {
    /// A description of the error that occurred.
    pub(crate) message: String,
    /// The kind of error that occurred.
    pub(crate) kind: InvalidEventKind,
}

/// The kind of error that occurred.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum InvalidEventKind {
    /// A deserialization error from malformed input.
    Deserialization,
    /// An error occurred validating input according the the matrix spec.
    Validation,
}

impl InvalidEvent {
    /// Constructor used in the event content macros.
    ///
    /// This has to be public to allow the macros to be used outside of ruma-events.
    #[doc(hidden)]
    pub fn wrong_event_type(expected: &str, found: &str) -> Self {
        Self {
            message: format!("expected `{}` found {}", expected, found),
            kind: InvalidEventKind::Deserialization,
        }
    }
    /// A message describing why the event is invalid.
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Returns whether this is a deserialization error.
    pub fn is_deserialization(&self) -> bool {
        self.kind == InvalidEventKind::Deserialization
    }

    /// Returns whether this is a validation error.
    pub fn is_validation(&self) -> bool {
        self.kind == InvalidEventKind::Validation
    }
}

impl Display for InvalidEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl Error for InvalidEvent {}

/// An error returned when attempting to create an event with data that would make it invalid.
///
/// This type is similar to [`InvalidEvent`](struct.InvalidEvent.html), but used during the
/// construction of a new event, as opposed to deserialization of an existing event from JSON.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidInput(pub(crate) String);

impl Display for InvalidInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for InvalidInput {}

/// An error when attempting to create a value from a string via the `FromStr` trait.
#[derive(Clone, Copy, Eq, Debug, Hash, PartialEq)]
pub struct FromStrError;

impl Display for FromStrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse type from string")
    }
}

impl Error for FromStrError {}
