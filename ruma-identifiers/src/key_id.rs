use std::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZeroU8,
    str::FromStr,
};

use crate::{crypto_algorithms::SigningKeyAlgorithm, DeviceId, Error, KeyName};

/// A key algorithm and key name delimited by a colon
pub struct KeyId<A, K: ?Sized> {
    full_id: Box<str>,
    colon_idx: NonZeroU8,
    _phantom: PhantomData<(A, K)>,
}

impl<A, K: ?Sized> KeyId<A, K> {
    /// Creates a `KeyId` from an algorithm and key name.
    pub fn from_parts(algorithm: A, key_name: &K) -> Self
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

        let colon_idx =
            NonZeroU8::new(algorithm.len().try_into().expect("no algorithm name len > 255"))
                .expect("no empty algorithm name");

        KeyId { full_id: res.into(), colon_idx, _phantom: PhantomData }
    }

    /// Returns key algorithm of the key ID.
    pub fn algorithm(&self) -> A
    where
        A: FromStr,
    {
        A::from_str(&self.full_id[..self.colon_idx.get() as usize])
            .unwrap_or_else(|_| unreachable!())
    }

    /// Returns the key name of the key ID.
    pub fn key_name<'a>(&'a self) -> &'a K
    where
        &'a K: From<&'a str>,
    {
        self.full_id[self.colon_idx.get() as usize + 1..].into()
    }
}

fn try_from<S, A, K: ?Sized>(key_identifier: S) -> Result<KeyId<A, K>, Error>
where
    S: AsRef<str> + Into<Box<str>>,
{
    let colon_idx = ruma_identifiers_validation::key_id::validate(key_identifier.as_ref())?;
    Ok(KeyId { full_id: key_identifier.into(), colon_idx, _phantom: PhantomData })
}

impl<A, K: ?Sized> KeyId<A, K> {
    /// Creates a string slice from this `KeyId<A, K>`
    pub fn as_str(&self) -> &str {
        &self.full_id
    }

    /// Creates a byte slice from this `KeyId<A, K>`
    pub fn as_bytes(&self) -> &[u8] {
        self.full_id.as_bytes()
    }
}

impl<A, K: ?Sized> Clone for KeyId<A, K> {
    fn clone(&self) -> Self {
        Self { full_id: self.full_id.clone(), colon_idx: self.colon_idx, _phantom: PhantomData }
    }
}

impl<A, K: ?Sized> AsRef<str> for KeyId<A, K> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<A, K: ?Sized> From<KeyId<A, K>> for String {
    fn from(id: KeyId<A, K>) -> Self {
        id.full_id.into()
    }
}

impl<A, K: ?Sized> FromStr for KeyId<A, K> {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        try_from(s)
    }
}

impl<A, K: ?Sized> TryFrom<&str> for KeyId<A, K> {
    type Error = crate::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl<A, K: ?Sized> TryFrom<String> for KeyId<A, K>
where
    A: FromStr,
    K: From<String>,
{
    type Error = crate::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        try_from(s)
    }
}

impl<A, K: ?Sized> fmt::Debug for KeyId<A, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Using struct debug format for consistency with other ID types.
        // FIXME: Change all ID types to have just a string debug format?
        f.debug_struct("KeyId")
            .field("full_id", &self.full_id)
            .field("colon_idxs", &self.colon_idx)
            .finish()
    }
}

impl<A, K: ?Sized> fmt::Display for KeyId<A, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
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
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<A, K: ?Sized> Ord for KeyId<A, K> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
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

#[cfg(feature = "serde")]
impl<'de, A, K: ?Sized> serde::Deserialize<'de> for KeyId<A, K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        crate::deserialize_id(deserializer, "Key name with algorithm and key identifier")
    }
}

#[rustfmt::skip]
partial_eq_string!(KeyId<A, K> [A, K]);

/// Algorithm + key name for signing keys.
pub type SigningKeyId<K> = KeyId<SigningKeyAlgorithm, K>;

/// Algorithm + key name for homeserver signing keys.
pub type ServerSigningKeyId = SigningKeyId<KeyName>;

/// Algorithm + key name for device keys.
pub type DeviceSigningKeyId = SigningKeyId<DeviceId>;
