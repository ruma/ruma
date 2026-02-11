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

ruma_common::priv_owned_str!(uniffi);
