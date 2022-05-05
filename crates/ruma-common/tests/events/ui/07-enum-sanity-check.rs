use ruma_common::events;
use ruma_macros::event_enum;

event_enum! {
    /// Any global account data event.
    enum GlobalAccountData {
        #[ruma_enum(alias = "io.ruma.direct")]
        "m.direct" => events::direct,
        #[ruma_enum(alias = "m.identity_server")]
        "io.ruma.identity_server" => events::identity_server,
        #[cfg(test)]
        "m.ignored_user_list" => events::ignored_user_list,
        // Doesn't actually have a wildcard, but this should work as a wildcard test
        "m.push_rules.*" => events::push_rules,
        #[cfg(any())]
        "m.ruma_test" => events::ruma_test,
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

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrivOwnedStr(Box<str>);
