use std::{
    clone::Clone,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use serde_json::value::RawValue;

use crate::{
    error::{InvalidEvent, InvalidEventKind},
    TryFromRaw,
};

/// A wrapper around `Box<RawValue>`, to be used in place of event [content] [collection] types in
/// Matrix endpoint definition to allow request and response types to contain unknown events in
/// addition to the known event(s) represented by the generic argument `Ev`.
pub struct EventJson<T> {
    json: Box<RawValue>,
    _ev: PhantomData<T>,
}

impl<T> EventJson<T> {
    fn new(json: Box<RawValue>) -> Self {
        Self {
            json,
            _ev: PhantomData,
        }
    }

    /// Access the underlying json value.
    pub fn json(&self) -> &RawValue {
        &self.json
    }

    /// Convert `self` into the underlying json value.
    pub fn into_json(self) -> Box<RawValue> {
        self.json
    }
}

impl<T: TryFromRaw> EventJson<T> {
    /// Try to deserialize the JSON into the expected event type.
    pub fn deserialize(&self) -> Result<T, InvalidEvent> {
        let raw_ev: T::Raw = match serde_json::from_str(self.json.get()) {
            Ok(raw) => raw,
            Err(error) => {
                return Err(InvalidEvent {
                    message: error.to_string(),
                    kind: InvalidEventKind::Deserialization,
                });
            }
        };

        match T::try_from_raw(raw_ev) {
            Ok(value) => Ok(value),
            Err(err) => Err(InvalidEvent {
                message: err.to_string(),
                kind: InvalidEventKind::Validation,
            }),
        }
    }
}

impl<T: Serialize> From<&T> for EventJson<T> {
    fn from(val: &T) -> Self {
        Self::new(serde_json::value::to_raw_value(val).unwrap())
    }
}

// Without the `TryFromRaw` bound, this would conflict with the next impl below
// We could remove the `TryFromRaw` bound once specialization is stabilized.
impl<T: Serialize + TryFromRaw> From<T> for EventJson<T> {
    fn from(val: T) -> Self {
        Self::from(&val)
    }
}

impl<T> From<Box<RawValue>> for EventJson<T> {
    fn from(json: Box<RawValue>) -> Self {
        Self::new(json)
    }
}

impl<T> Clone for EventJson<T> {
    fn clone(&self) -> Self {
        Self::new(self.json.clone())
    }
}

impl<T> Debug for EventJson<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use std::any::type_name;
        f.debug_struct(&format!("EventJson::<{}>", type_name::<T>()))
            .field("json", &self.json)
            .finish()
    }
}

impl<'de, T> Deserialize<'de> for EventJson<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Box::<RawValue>::deserialize(deserializer).map(Self::new)
    }
}

impl<T> Serialize for EventJson<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.json.serialize(serializer)
    }
}
