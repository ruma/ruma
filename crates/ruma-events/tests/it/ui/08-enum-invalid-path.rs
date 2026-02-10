#![allow(unexpected_cfgs)]

#[cfg(feature = "unstable-uniffi")]
uniffi::setup_scaffolding!();

use ruma_macros::event_enum;

event_enum! {
    enum RoomAccountData {
        "m.not.a.path" => ruma_events::not::a::path,
    }
}

fn main() {}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrivOwnedStr(Box<str>);

#[cfg_attr(feature = "unstable-uniffi", derive(uniffi::Object))]
#[doc(hidden)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrivateString(Box<str>);

#[cfg(feature = "unstable-uniffi")]
uniffi::custom_type!(PrivOwnedStr, std::sync::Arc<PrivateString> , {
    lower: |value| std::sync::Arc::new(PrivateString(value.0)),
    try_lift: |value| Ok(PrivOwnedStr(value.0.clone())),
});
