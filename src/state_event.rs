use ruma::events::{
    from_raw_json_value, AnyStateEvent, AnyStrippedStateEvent, AnySyncStateEvent, EventDeHelper,
};
use serde::{de, Serialize};
use serde_json::value::RawValue as RawJsonValue;

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum StateEvent {
    Full(AnyStateEvent),
    Sync(AnySyncStateEvent),
    Stripped(AnyStrippedStateEvent),
}

impl<'de> de::Deserialize<'de> for StateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let json = Box::<RawJsonValue>::deserialize(deserializer)?;
        let EventDeHelper {
            state_key,
            event_id,
            room_id,
            unsigned,
            ..
        } = from_raw_json_value(&json)?;

        // Determine whether the event is a full, sync, or stripped
        // based on the fields present.
        if room_id.is_some() {
            Ok(match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    panic!("TODO deal with redacted events")
                }
                _ => StateEvent::Full(from_raw_json_value(&json)?),
            })
        } else if event_id.is_some() {
            Ok(match unsigned {
                Some(unsigned) if unsigned.redacted_because.is_some() => {
                    panic!("TODO deal with redacted events")
                }
                _ => StateEvent::Sync(from_raw_json_value(&json)?),
            })
        } else {
            Ok(StateEvent::Stripped(from_raw_json_value(&json)?))
        }
    }
}
