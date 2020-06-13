use ruma_events_macros::AnyEventDeserialize;

#[derive(Clone, Debug, AnyEventDeserialize)]
pub enum AnyRoomEventStub {
    Message(()),
    Redaction(()),
    StateEvent(()),
}

fn main() {}
