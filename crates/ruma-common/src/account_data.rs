//! (De)serializable types for user- and room-scoped account data, used by the client-server API.

use ruma_macros::{account_data_enum, AccountData};
use serde::Serialize;
use serde_json::value::RawValue as RawJsonValue;

pub mod direct;
pub mod fully_read;
pub mod ignored_user_list;
pub mod push_rules;
pub mod tag;

// Needs to be public for trybuild tests
#[doc(hidden)]
pub mod _custom;

/// The base trait that all account data content types implement.
///
/// Implementing this trait allows content types to be serialized as well as deserialized.
pub trait AccountDataContent: Sized + Serialize {
    /// The type, like `m.tag`.
    fn data_type(&self) -> &str;

    /// Constructs the given content.
    fn from_parts(data_type: &str, content: &RawJsonValue) -> serde_json::Result<Self>;
}

/// Marker trait for the content of a global account data object.
pub trait GlobalAccountDataContent: AccountDataContent {}

/// Marker trait for the content of a room account data object.
pub trait RoomAccountDataContent: AccountDataContent {}

/// Global account data.
#[derive(Clone, Debug, AccountData)]
#[allow(clippy::exhaustive_structs)]
pub struct GlobalAccountData<C: GlobalAccountDataContent> {
    /// Data specific to the account data type.
    pub content: C,
}

/// Room-bound account data.
#[derive(Clone, Debug, AccountData)]
#[allow(clippy::exhaustive_structs)]
pub struct RoomAccountData<C: RoomAccountDataContent> {
    /// Data specific to the account data type.
    pub content: C,
}

account_data_enum! {
    /// Any global account data object.
    enum GlobalAccountData {
        "m.direct",
        "m.ignored_user_list",
        "m.push_rules",
    }
}

account_data_enum! {
    /// Any room account data object.
    enum RoomAccountData {
        "m.fully_read",
        "m.tag",
    }
}
