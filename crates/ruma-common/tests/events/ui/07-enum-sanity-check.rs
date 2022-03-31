use ruma_macros::event_enum;

event_enum! {
    /// Any global account data event.
    enum GlobalAccountData {
        "m.direct",
        #[cfg(test)]
        "m.ignored_user_list",
        // Doesn't actually have a wildcard, but this should work as a wildcard test
        "m.push_rules.*",
        #[cfg(any())]
        "m.ruma_test",
    }
}

fn main() {}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrivOwnedStr(Box<str>);
