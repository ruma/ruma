//! Common types for other ruma crates.

#![warn(missing_docs, missing_debug_implementations)]

pub mod directory;
pub mod encryption;
pub mod presence;
pub mod push;
mod raw;
pub mod thirdparty;

pub use ruma_common_macros::Outgoing;

pub use self::raw::Raw;

/// A type that can be sent to another party that understands the matrix protocol. If any of the
/// fields of `Self` don't implement serde's `Deserialize`, you can derive this trait to generate a
/// corresponding 'Incoming' type that supports deserialization. This is useful for things like
/// ruma_events' `EventResult` type. For more details, see the [derive macro's documentation][doc].
///
/// [doc]: derive.Outgoing.html
// TODO: Better explain how this trait relates to serde's traits
pub trait Outgoing {
    /// The 'Incoming' variant of `Self`.
    type Incoming;
}

// Hack to allow both ruma-common itself and external crates (or tests) to use procedural macros
// that expect `ruma_common` to exist in the prelude.
extern crate self as ruma_common;

/// This module is used to support the generated code from ruma-api-macros.
/// It is not considered part of ruma-common's public API.
#[doc(hidden)]
pub mod exports {
    pub use serde;
}
