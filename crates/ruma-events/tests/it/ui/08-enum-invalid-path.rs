#![allow(unexpected_cfgs)]

use ruma_macros::event_enum;

event_enum! {
    enum State {
        "m.not.a.path" => ruma_events::not::a::path,
    }
}

fn main() {}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrivOwnedStr(Box<str>);
