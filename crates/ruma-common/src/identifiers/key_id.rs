use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    marker::PhantomData,
    str::FromStr,
};

use ruma_macros::IdZst;

use super::{crypto_algorithms::SigningKeyAlgorithm, DeviceId, KeyName};

/// A key algorithm and key name delimited by a colon.
#[repr(transparent)]
#[derive(IdZst)]
#[ruma_id(validate = ruma_identifiers_validation::key_id::validate)]
pub struct KeyId<A, K: ?Sized>(PhantomData<(A, K)>, str);

impl<A, K: ?Sized> KeyId<A, K> {
    /// Creates a new `KeyId` from an algorithm and key name.
    pub fn from_parts(algorithm: A, key_name: &K) -> OwnedKeyId<A, K>
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

        Self::from_borrowed(&res).to_owned()
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

    fn colon_idx(&self) -> usize {
        self.as_str().find(':').unwrap()
    }
}

/// Algorithm + key name for signing keys.
pub type SigningKeyId<K> = KeyId<SigningKeyAlgorithm, K>;

/// Algorithm + key name for signing keys.
pub type OwnedSigningKeyId<K> = OwnedKeyId<SigningKeyAlgorithm, K>;

/// Algorithm + key name for homeserver signing keys.
pub type ServerSigningKeyId = SigningKeyId<KeyName>;

/// Algorithm + key name for homeserver signing keys.
pub type OwnedServerSigningKeyId = OwnedSigningKeyId<KeyName>;

/// Algorithm + key name for device keys.
pub type DeviceSigningKeyId = SigningKeyId<DeviceId>;

/// Algorithm + key name for device keys.
pub type OwnedDeviceSigningKeyId = OwnedSigningKeyId<DeviceId>;

// The following impls are usually derived using the std macros.
// They are implemented manually here to avoid unnecessary bounds.
impl<A, K: ?Sized> PartialEq for KeyId<A, K> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A, K: ?Sized> Eq for KeyId<A, K> {}

impl<A, K: ?Sized> PartialOrd for KeyId<A, K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
