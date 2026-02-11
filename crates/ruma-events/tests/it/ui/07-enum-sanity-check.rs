#![allow(unexpected_cfgs)]

#[cfg(feature = "unstable-uniffi")]
uniffi::setup_scaffolding!();

use ruma_macros::event_enum;

event_enum! {
    /// Any global account data event.
    enum GlobalAccountData {
        #[ruma_enum(alias = "io.ruma.direct")]
        "m.direct" => ruma_events::direct,
        #[ruma_enum(alias = "m.identity_server")]
        "io.ruma.identity_server" => ruma_events::identity_server,
        #[cfg(test)]
        "m.ignored_user_list" => ruma_events::ignored_user_list,
        // Doesn't actually have a wildcard, but this should work as a wildcard test
        "m.push_rules.*" => ruma_events::push_rules,
        #[cfg(any())]
        "m.ruma_test" => ruma_events::ruma_test,
    }
}

fn main() {
    assert_eq!(GlobalAccountDataEventType::from("m.direct"), GlobalAccountDataEventType::Direct);
    assert_eq!(
        GlobalAccountDataEventType::from("io.ruma.direct"),
        GlobalAccountDataEventType::Direct
    );
    assert_eq!(GlobalAccountDataEventType::Direct.to_cow_str(), "m.direct");

    assert_eq!(
        GlobalAccountDataEventType::from("m.identity_server"),
        GlobalAccountDataEventType::IdentityServer
    );
    assert_eq!(
        GlobalAccountDataEventType::from("io.ruma.identity_server"),
        GlobalAccountDataEventType::IdentityServer
    );
    assert_eq!(GlobalAccountDataEventType::IdentityServer.to_cow_str(), "io.ruma.identity_server");
}

ruma_common::priv_owned_str!(uniffi);
