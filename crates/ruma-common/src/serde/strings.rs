use std::{collections::BTreeMap, fmt, marker::PhantomData};

use js_int::{Int, UInt};
use serde::{
    de::{self, Deserializer, IntoDeserializer as _, MapAccess, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};

/// Serde deserialization decorator to map empty Strings to None,
/// and forward non-empty Strings to the Deserialize implementation for T.
/// Useful for the typical
/// "A room with an X event with an absent, null, or empty Y field
/// should be treated the same as a room with no such event."
/// formulation in the spec.
///
/// To be used like this:
/// `#[serde(default, deserialize_with = "empty_string_as_none")]`
/// Relevant serde issue: <https://github.com/serde-rs/serde/issues/1425>
pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        // If T = String, like in m.room.name, the second deserialize is actually superfluous.
        // TODO: optimize that somehow?
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}

/// Serde serializiation decorator to map `None` to an empty `String`,
/// and forward `Some`s to the `Serialize` implementation for `T`.
///
/// To be used like this:
/// `#[serde(serialize_with = "empty_string_as_none")]`
pub fn none_as_empty_string<T: Serialize, S>(
    value: &Option<T>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(x) => x.serialize(serializer),
        None => serializer.serialize_str(""),
    }
}

/// Take either an integer number or a string and deserialize to an integer number.
///
/// To be used like this:
/// `#[serde(deserialize_with = "deserialize_v1_powerlevel")]`
pub fn deserialize_v1_powerlevel<'de, D>(de: D) -> Result<Int, D::Error>
where
    D: Deserializer<'de>,
{
    struct IntOrStringVisitor;

    impl<'de> Visitor<'de> for IntOrStringVisitor {
        type Value = Int;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an integer or a string")
        }

        fn visit_i8<E: de::Error>(self, v: i8) -> Result<Self::Value, E> {
            Ok(v.into())
        }

        fn visit_i16<E: de::Error>(self, v: i16) -> Result<Self::Value, E> {
            Ok(v.into())
        }

        fn visit_i32<E: de::Error>(self, v: i32) -> Result<Self::Value, E> {
            Ok(v.into())
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            v.try_into().map_err(E::custom)
        }

        fn visit_i128<E: de::Error>(self, v: i128) -> Result<Self::Value, E> {
            v.try_into().map_err(E::custom)
        }

        fn visit_u8<E: de::Error>(self, v: u8) -> Result<Self::Value, E> {
            Ok(v.into())
        }

        fn visit_u16<E: de::Error>(self, v: u16) -> Result<Self::Value, E> {
            Ok(v.into())
        }

        fn visit_u32<E: de::Error>(self, v: u32) -> Result<Self::Value, E> {
            Ok(v.into())
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            v.try_into().map_err(E::custom)
        }

        fn visit_u128<E: de::Error>(self, v: u128) -> Result<Self::Value, E> {
            v.try_into().map_err(E::custom)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            let trimmed = v.trim();

            match trimmed.strip_prefix('+') {
                Some(without) => without.parse::<UInt>().map(|u| u.into()).map_err(E::custom),
                None => trimmed.parse().map_err(E::custom),
            }
        }
    }

    de.deserialize_any(IntOrStringVisitor)
}

/// Take a BTreeMap with values of either an integer number or a string and deserialize
/// those to integer numbers.
///
/// To be used like this:
/// `#[serde(deserialize_with = "btreemap_deserialize_v1_powerlevel_values")]`
pub fn btreemap_deserialize_v1_powerlevel_values<'de, D, T>(
    de: D,
) -> Result<BTreeMap<T, Int>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Ord,
{
    #[repr(transparent)]
    struct IntWrap(Int);

    impl<'de> Deserialize<'de> for IntWrap {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize_v1_powerlevel(deserializer).map(IntWrap)
        }
    }

    struct IntMapVisitor<T> {
        _phantom: PhantomData<T>,
    }

    impl<T> IntMapVisitor<T> {
        fn new() -> Self {
            Self { _phantom: PhantomData }
        }
    }

    impl<'de, T> Visitor<'de> for IntMapVisitor<T>
    where
        T: Deserialize<'de> + Ord,
    {
        type Value = BTreeMap<T, Int>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map with integers or stings as values")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut res = BTreeMap::new();

            while let Some((k, IntWrap(v))) = map.next_entry()? {
                res.insert(k, v);
            }

            Ok(res)
        }
    }

    de.deserialize_map(IntMapVisitor::new())
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use js_int::{int, Int};
    use serde::Deserialize;

    use super::deserialize_v1_powerlevel;

    #[derive(Debug, Deserialize)]
    struct Test {
        #[serde(deserialize_with = "deserialize_v1_powerlevel")]
        num: Int,
    }

    #[test]
    fn int_or_string() -> serde_json::Result<()> {
        assert_matches!(
            serde_json::from_value::<Test>(serde_json::json!({ "num": "0" }))?,
            Test { num } if num == int!(0)
        );

        Ok(())
    }

    #[test]
    fn weird_plus_string() -> serde_json::Result<()> {
        assert_matches!(
            serde_json::from_value::<Test>(serde_json::json!({ "num": "  +0000000001000   " }))?,
            Test { num } if num == int!(1000)
        );

        Ok(())
    }

    #[test]
    fn weird_minus_string() -> serde_json::Result<()> {
        assert_matches!(
            serde_json::from_value::<Test>(serde_json::json!({ "num": "  \n\n-0000000000000001000   " }))?,
            Test { num } if num == int!(-1000)
        );

        Ok(())
    }
}
