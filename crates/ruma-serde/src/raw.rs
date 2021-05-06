use std::{
    clone::Clone,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use serde::{
    de::{Deserialize, DeserializeOwned, Deserializer, IgnoredAny, MapAccess, Visitor},
    ser::{Serialize, Serializer},
};
use serde_json::value::RawValue;

use crate::cow::MyCowStr;

/// A wrapper around `Box<RawValue>`, to be used in place of any type in the Matrix endpoint
/// definition to allow request and response types to contain that said type represented by
/// the generic argument `Ev`.
///
/// Ruma offers the `Raw` wrapper to enable passing around JSON text that is only partially
/// validated. This is useful when a client receives events that do not follow the spec perfectly
/// or a server needs to generate reference hashes with the original canonical JSON string.
/// All event structs and enums implement `Serialize` / `Deserialize`, `Raw` should be used
/// to pass around events in a lossless way.
///
/// ```ignore
/// let json = r#"{ "type": "imagine a full event", "content": {...} }"#;
///
/// let deser = serde_json::from_str::<Raw<AnyRoomEvent>>(json)
///     .unwrap() // the first Result from serde_json::from_str, will not fail
///     .deserialize() // deserialize to the inner type
///     .unwrap(); // finally get to the AnyRoomEvent
/// ```
pub struct Raw<T> {
    json: Box<RawValue>,
    _ev: PhantomData<T>,
}

impl<T> Raw<T> {
    fn new(json: Box<RawValue>) -> Self {
        Self { json, _ev: PhantomData }
    }

    /// Create a `Raw` from a boxed `RawValue`.
    pub fn from_json(raw: Box<RawValue>) -> Self {
        Self::new(raw)
    }

    /// Access the underlying json value.
    pub fn json(&self) -> &RawValue {
        &self.json
    }

    /// Convert `self` into the underlying json value.
    pub fn into_json(self) -> Box<RawValue> {
        self.json
    }

    /// Try to access a given field inside this `Raw`, assuming it contains an
    /// object.
    ///
    /// Returns `Err(_)` when the contained value is not an object, or the field
    /// exists but is fails to deserialize to the expected type.
    ///
    /// Returns `Ok(None)` when the field doesn't exist or is `null`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # type CustomMatrixEvent = ();
    /// # fn foo() -> serde_json::Result<()> {
    /// # let raw_event: ruma_serde::Raw<()> = todo!();
    /// if raw_event.get_field::<String>("type")?.as_deref() == Some("org.custom.matrix.event") {
    ///     let event: CustomMatrixEvent = serde_json::from_str(raw_event.json().get())?;
    ///     // ...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_field<'a, U>(&'a self, field_name: &str) -> serde_json::Result<Option<U>>
    where
        U: Deserialize<'a>,
    {
        struct SingleFieldVisitor<'b, T> {
            field_name: &'b str,
            _phantom: PhantomData<T>,
        }

        impl<'b, T> SingleFieldVisitor<'b, T> {
            fn new(field_name: &'b str) -> Self {
                Self { field_name, _phantom: PhantomData }
            }
        }

        impl<'b, 'de, T> Visitor<'de> for SingleFieldVisitor<'b, T>
        where
            T: Deserialize<'de>,
        {
            type Value = Option<T>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut res = None;
                while let Some(key) = map.next_key::<MyCowStr<'_>>()? {
                    if key.get() == self.field_name {
                        res = Some(map.next_value()?);
                    } else {
                        map.next_value::<IgnoredAny>()?;
                    }
                }

                Ok(res)
            }
        }

        let mut deserializer = serde_json::Deserializer::from_str(self.json().get());
        deserializer.deserialize_map(SingleFieldVisitor::new(field_name))
    }
}

impl<T> Raw<T>
where
    T: DeserializeOwned,
{
    /// Try to deserialize the JSON into the expected type.
    pub fn deserialize(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(self.json.get())
    }
}

impl<T: Serialize> From<&T> for Raw<T> {
    fn from(val: &T) -> Self {
        Self::new(serde_json::value::to_raw_value(val).unwrap())
    }
}

// With specialization a fast path from impl for `impl<T> From<Box<RawValue...`
// could be used. Until then there is a special constructor `from_json` for this.
impl<T: Serialize> From<T> for Raw<T> {
    fn from(val: T) -> Self {
        Self::from(&val)
    }
}

impl<T> Clone for Raw<T> {
    fn clone(&self) -> Self {
        Self::new(self.json.clone())
    }
}

impl<T> Debug for Raw<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use std::any::type_name;
        f.debug_struct(&format!("Raw::<{}>", type_name::<T>())).field("json", &self.json).finish()
    }
}

impl<'de, T> Deserialize<'de> for Raw<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Box::<RawValue>::deserialize(deserializer).map(Self::new)
    }
}

impl<T> Serialize for Raw<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.json.serialize(serializer)
    }
}
