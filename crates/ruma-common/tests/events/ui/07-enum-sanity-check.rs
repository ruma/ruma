use ruma_macros::event_enum;

event_enum! {
    /// Any state event.
    enum State {
        "m.room.aliases",
        #[cfg(test)]
        "m.room.avatar",
        "m.room.canonical_alias",
        "m.room.create",
        #[cfg(any())]
        "m.ruma_test",
    }
}

fn main() {}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrivOwnedStr(Box<str>);
