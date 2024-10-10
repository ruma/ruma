use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use ruma_macros::IdZst;

use super::{
    crypto_algorithms::SigningKeyAlgorithm, DeviceId, KeyName, OneTimeKeyAlgorithm, OneTimeKeyName,
    ServerSigningKeyVersion,
};

/// A key algorithm and key name delimited by a colon.
#[repr(transparent)]
#[derive(IdZst)]
#[ruma_id(
    validate = ruma_identifiers_validation::key_id::validate::<K>,
)]
pub struct KeyId<A: KeyAlgorithm, K: KeyName + ?Sized>(PhantomData<(A, K)>, str);

impl<A: KeyAlgorithm, K: KeyName + ?Sized> KeyId<A, K> {
    /// Creates a new `KeyId` from an algorithm and key name.
    pub fn from_parts(algorithm: A, key_name: &K) -> OwnedKeyId<A, K> {
        let algorithm = algorithm.as_ref();
        let key_name = key_name.as_ref();

        let mut res = String::with_capacity(algorithm.len() + 1 + key_name.len());
        res.push_str(algorithm);
        res.push(':');
        res.push_str(key_name);

        Self::from_borrowed(&res).to_owned()
    }

    /// Returns key algorithm of the key ID.
    pub fn algorithm(&self) -> A {
        A::from(&self.as_str()[..self.colon_idx()])
    }

    /// Returns the key name of the key ID.
    pub fn key_name<'a>(&'a self) -> &'a K
    where
        &'a K: TryFrom<&'a str>,
    {
        <&'a K>::try_from(&self.as_str()[..self.colon_idx()]).unwrap_or_else(|_| unreachable!())
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
pub type ServerSigningKeyId = SigningKeyId<ServerSigningKeyVersion>;

/// Algorithm + key name for homeserver signing keys.
pub type OwnedServerSigningKeyId = OwnedSigningKeyId<ServerSigningKeyVersion>;

/// Algorithm + key name for device keys.
pub type DeviceSigningKeyId = SigningKeyId<DeviceId>;

/// Algorithm + key name for device keys.
pub type OwnedDeviceSigningKeyId = OwnedSigningKeyId<DeviceId>;

/// Algorithm + key name for [one-time and fallback keys].
///
/// [one-time and fallback keys]: https://spec.matrix.org/latest/client-server-api/#one-time-and-fallback-keys
pub type OneTimeKeyId = KeyId<OneTimeKeyAlgorithm, OneTimeKeyName>;

/// Algorithm + key name for [one-time and fallback keys].
///
/// [one-time and fallback keys]: https://spec.matrix.org/latest/client-server-api/#one-time-and-fallback-keys
pub type OwnedOneTimeKeyId = OwnedKeyId<OneTimeKeyAlgorithm, OneTimeKeyName>;

// The following impls are usually derived using the std macros.
// They are implemented manually here to avoid unnecessary bounds.
impl<A: KeyAlgorithm, K: KeyName + ?Sized> PartialEq for KeyId<A, K> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<A: KeyAlgorithm, K: KeyName + ?Sized> Eq for KeyId<A, K> {}

impl<A: KeyAlgorithm, K: KeyName + ?Sized> PartialOrd for KeyId<A, K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: KeyAlgorithm, K: KeyName + ?Sized> Ord for KeyId<A, K> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(self.as_str(), other.as_str())
    }
}

impl<A: KeyAlgorithm, K: KeyName + ?Sized> Hash for KeyId<A, K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

/// The algorithm of a key.
pub trait KeyAlgorithm: for<'a> From<&'a str> + AsRef<str> {}

impl KeyAlgorithm for SigningKeyAlgorithm {}

impl KeyAlgorithm for OneTimeKeyAlgorithm {}
