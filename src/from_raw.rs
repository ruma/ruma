use std::{collections::BTreeMap, convert::Infallible, fmt::Display};

use serde::de::DeserializeOwned;

/// See [`TryFromRaw`][try]. This trait is merely a convenience that is be implemented instead of
/// [`TryFromRaw`][try] to get a [`TryFromRaw`][try] implementation with slightly less code if the
/// conversion can't fail, that is, the raw type and `Self` are identical in definition.
///
/// [try]: trait.TryFromRaw.html
pub trait FromRaw: Sized {
    /// The raw type.
    type Raw: DeserializeOwned;

    /// Converts the raw type to `Self`.
    fn from_raw(_: Self::Raw) -> Self;
}

/// Types corresponding to some item in the matrix spec. Types that implement this trait have a
/// corresponding 'raw' type, a potentially invalid representation that can be converted to `Self`.
pub trait TryFromRaw: Sized {
    /// The raw type.
    type Raw: DeserializeOwned;
    /// The error type returned if conversion fails.
    type Err: Display;

    /// Tries to convert the raw type to `Self`.
    fn try_from_raw(_: Self::Raw) -> Result<Self, Self::Err>;
}

impl FromRaw for serde_json::Value {
    type Raw = Self;

    fn from_raw(raw: Self) -> Self {
        raw
    }
}

impl<K, V> FromRaw for BTreeMap<K, V>
where
    Self: DeserializeOwned,
{
    type Raw = Self;

    fn from_raw(raw: Self) -> Self {
        raw
    }
}

impl<T: FromRaw> TryFromRaw for T {
    type Raw = <T as FromRaw>::Raw;
    type Err = Infallible;

    fn try_from_raw(raw: Self::Raw) -> Result<Self, Self::Err> {
        Ok(Self::from_raw(raw))
    }
}
