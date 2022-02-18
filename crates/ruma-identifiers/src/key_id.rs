use std::{
    cmp::Ordering,
    convert::TryFrom,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    rc::Rc,
    str::FromStr,
    sync::Arc,
};

use crate::{crypto_algorithms::SigningKeyAlgorithm, DeviceId, KeyName};

/// A key algorithm and key name delimited by a colon.
#[repr(transparent)]
pub struct KeyId<A, K: ?Sized>(PhantomData<(A, K)>, str);

impl<A, K: ?Sized> KeyId<A, K> {
    /// Creates a new `KeyId` from an algorithm and key name.
    pub fn from_parts(algorithm: A, key_name: &K) -> Box<Self>
    where
        A: AsRef<str>,
        K: AsRef<str>,
    {
        let algorithm = algorithm.as_ref();
        let key_name = key_name.as_ref();

        let mut res = String::with_capacity(algorithm.len() + 1 + key_name.len());
        res.push_str(algorithm);
        res.push(':');
        res.push_str(key_name);

        Self::from_owned(res.into())
    }

    /// Returns key algorithm of the key ID.
    pub fn algorithm(&self) -> A
    where
        A: FromStr,
    {
        A::from_str(&self.as_str()[..self.colon_idx()]).unwrap_or_else(|_| unreachable!())
    }

    /// Returns the key name of the key ID.
    pub fn key_name<'a>(&'a self) -> &'a K
    where
        &'a K: From<&'a str>,
    {
        self.as_str()[self.colon_idx() + 1..].into()
    }

    /// Creates a string slice from this `KeyId`.
    pub fn as_str(&self) -> &str {
        &self.1
    }

    /// Creates a byte slice from this `KeyId`.
    pub fn as_bytes(&self) -> &[u8] {
        self.1.as_bytes()
    }

    fn from_borrowed(s: &str) -> &Self {
        unsafe { std::mem::transmute(s) }
    }

    fn from_owned(s: Box<str>) -> Box<Self> {
        unsafe { Box::from_raw(Box::into_raw(s) as _) }
    }

    fn into_owned(self: Box<Self>) -> Box<str> {
        unsafe { Box::from_raw(Box::into_raw(self) as _) }
    }

    fn colon_idx(&self) -> usize {
        self.as_str().find(':').unwrap()
    }
}

/// Algorithm + key name for signing keys.
pub type SigningKeyId<K> = KeyId<SigningKeyAlgorithm, K>;

/// Algorithm + key name for homeserver signing keys.
pub type ServerSigningKeyId = SigningKeyId<KeyName>;

/// Algorithm + key name for device keys.
pub type DeviceSigningKeyId = SigningKeyId<DeviceId>;

impl<A, K: ?Sized> Clone for Box<KeyId<A, K>> {
    fn clone(&self) -> Self {
        (**self).to_owned()
    }
}

impl<A, K: ?Sized> ToOwned for KeyId<A, K> {
    type Owned = Box<KeyId<A, K>>;

    fn to_owned(&self) -> Self::Owned {
        Self::from_owned(self.1.into())
    }
}

impl<A, K: ?Sized> From<&KeyId<A, K>> for Box<KeyId<A, K>> {
    fn from(id: &KeyId<A, K>) -> Self {
        id.to_owned()
    }
}

impl<A, K: ?Sized> AsRef<str> for Box<KeyId<A, K>> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<A, K: ?Sized> From<&KeyId<A, K>> for Rc<KeyId<A, K>> {
    fn from(s: &KeyId<A, K>) -> Rc<KeyId<A, K>> {
        let rc = Rc::<str>::from(s.as_str());
        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const KeyId<A, K>) }
    }
}

impl<A, K: ?Sized> From<&KeyId<A, K>> for Arc<KeyId<A, K>> {
    fn from(s: &KeyId<A, K>) -> Arc<KeyId<A, K>> {
        let arc = Arc::<str>::from(s.as_str());
        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const KeyId<A, K>) }
    }
}

impl<A, K: ?Sized> PartialEq<KeyId<A, K>> for Box<KeyId<A, K>> {
    fn eq(&self, other: &KeyId<A, K>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A, K: ?Sized> PartialEq<&'_ KeyId<A, K>> for Box<KeyId<A, K>> {
    fn eq(&self, other: &&KeyId<A, K>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A, K: ?Sized> PartialEq<Box<KeyId<A, K>>> for KeyId<A, K> {
    fn eq(&self, other: &Box<KeyId<A, K>>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A, K: ?Sized> PartialEq<Box<KeyId<A, K>>> for &'_ KeyId<A, K> {
    fn eq(&self, other: &Box<KeyId<A, K>>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A, K: ?Sized> AsRef<str> for KeyId<A, K> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<A, K: ?Sized> fmt::Display for KeyId<A, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<A, K: ?Sized> fmt::Debug for KeyId<A, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<A, K: ?Sized> PartialEq for KeyId<A, K> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A, K: ?Sized> Eq for KeyId<A, K> {}

impl<A, K: ?Sized> PartialOrd for KeyId<A, K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(self.as_str(), other.as_str())
    }
}

impl<A, K: ?Sized> Ord for KeyId<A, K> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(self.as_str(), other.as_str())
    }
}

impl<A, K: ?Sized> Hash for KeyId<A, K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}
#[cfg(feature = "serde")]
impl<A, K: ?Sized> serde::Serialize for KeyId<A, K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<A, K: ?Sized> From<Box<KeyId<A, K>>> for String {
    fn from(id: Box<KeyId<A, K>>) -> Self {
        id.into_owned().into()
    }
}

#[cfg(feature = "serde")]
impl<'de, A, K: ?Sized> serde::Deserialize<'de> for Box<KeyId<A, K>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        match try_from(s) {
            Ok(o) => Ok(o),
            Err(e) => Err(D::Error::custom(e)),
        }
    }
}

fn try_from<S, A, K: ?Sized>(s: S) -> Result<Box<KeyId<A, K>>, crate::Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    ruma_identifiers_validation::key_id::validate(s.as_ref())?;
    Ok(KeyId::from_owned(s.into()))
}

impl<'a, A, K: ?Sized> TryFrom<&'a str> for &'a KeyId<A, K> {
    type Error = crate::Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        (ruma_identifiers_validation::key_id::validate)(s)?;
        Ok(KeyId::from_borrowed(s))
    }
}

impl<A, K: ?Sized> FromStr for Box<KeyId<A, K>> {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        try_from(s)
    }
}

impl<A, K: ?Sized> TryFrom<&str> for Box<KeyId<A, K>> {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl<A, K: ?Sized> TryFrom<String> for Box<KeyId<A, K>> {
    type Error = crate::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

#[rustfmt::skip]
partial_eq_string!(KeyId<A, K> [A, K]);
