use serde::Serialize;
use serde_json::value::RawValue as RawJsonValue;

use super::{AccountDataContent, GlobalAccountDataContent, RoomAccountDataContent};

/// A custom account data's type. Used for enum `_Custom` variants.
// FIXME: Serialize shouldn't be required here, but it's currently a supertrait of
// AccountDataContent
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::exhaustive_structs)]
pub struct CustomAccountDataContent {
    #[serde(skip)]
    data_type: Box<str>,
}

impl AccountDataContent for CustomAccountDataContent {
    fn data_type(&self) -> &str {
        &self.data_type
    }

    fn from_parts(data_type: &str, _content: &RawJsonValue) -> serde_json::Result<Self> {
        Ok(Self { data_type: data_type.into() })
    }
}

impl GlobalAccountDataContent for CustomAccountDataContent {}
impl RoomAccountDataContent for CustomAccountDataContent {}
