use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use super::{
    Base64PublicKeyOrDeviceId, DeviceId, KeyName, ServerName, ServerSigningKeyVersion,
    SigningKeyId, UserId,
};

/// Map of key identifier to signature values.
pub type EntitySignatures<K> = BTreeMap<SigningKeyId<K>, String>;

/// Map of all signatures, grouped by entity.
///
/// ```
/// # use ruma_common::{server_name, owned_server_signing_key_version, ServerSigningKeyId, Signatures, SigningKeyAlgorithm};
/// let key_identifier = ServerSigningKeyId::from_parts(
///     SigningKeyAlgorithm::Ed25519,
///     &owned_server_signing_key_version!("1")
/// );
/// let mut signatures = Signatures::new();
/// let server_name = server_name!("example.org");
/// let signature =
///     "YbJva03ihSj5mPk+CHMJKUKlCXCPFXjXOK6VqBnN9nA2evksQcTGn6hwQfrgRHIDDXO2le49x7jnWJHMJrJoBQ";
/// signatures.insert_signature(server_name, key_identifier, signature.into());
/// ```
#[derive(Debug, Serialize, Deserialize)]
#[serde(
    transparent,
    bound(serialize = "E: Serialize", deserialize = "E: serde::de::DeserializeOwned")
)]
pub struct Signatures<E: Ord, K: KeyName + ?Sized>(BTreeMap<E, EntitySignatures<K>>);

impl<E: Ord, K: KeyName + ?Sized> Signatures<E, K> {
    /// Creates an empty signature map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a signature for the given entity and key identifier.
    ///
    /// If there was already one, it is returned.
    pub fn insert_signature(
        &mut self,
        entity: E,
        key_identifier: SigningKeyId<K>,
        value: String,
    ) -> Option<String> {
        self.0.entry(entity).or_default().insert(key_identifier, value)
    }
}

/// Map of server signatures, grouped by server.
pub type ServerSignatures = Signatures<ServerName, ServerSigningKeyVersion>;

/// Map of device signatures, grouped by user.
pub type DeviceSignatures = Signatures<UserId, DeviceId>;

/// Map of cross-signing or device signatures, grouped by user.
pub type CrossSigningOrDeviceSignatures = Signatures<UserId, Base64PublicKeyOrDeviceId>;

impl<E, K> Clone for Signatures<E, K>
where
    E: Ord + Clone,
    K: KeyName + ?Sized,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<E: Ord, K: KeyName + ?Sized> Default for Signatures<E, K> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<E: Ord, K: KeyName + ?Sized> Deref for Signatures<E, K> {
    type Target = BTreeMap<E, EntitySignatures<K>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Ord, K: KeyName + ?Sized> DerefMut for Signatures<E, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E: Ord, K: KeyName + ?Sized, const N: usize> From<[(E, SigningKeyId<K>, String); N]>
    for Signatures<E, K>
{
    fn from(value: [(E, SigningKeyId<K>, String); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<E: Ord, K: KeyName + ?Sized> FromIterator<(E, SigningKeyId<K>, String)> for Signatures<E, K> {
    fn from_iter<T: IntoIterator<Item = (E, SigningKeyId<K>, String)>>(iter: T) -> Self {
        iter.into_iter().fold(Self::new(), |mut acc, (entity, key_identifier, value)| {
            acc.insert_signature(entity, key_identifier, value);
            acc
        })
    }
}

impl<E: Ord, K: KeyName + ?Sized> Extend<(E, SigningKeyId<K>, String)> for Signatures<E, K> {
    fn extend<T: IntoIterator<Item = (E, SigningKeyId<K>, String)>>(&mut self, iter: T) {
        for (entity, key_identifier, value) in iter {
            self.insert_signature(entity, key_identifier, value);
        }
    }
}

impl<E: Ord + Clone, K: KeyName + ?Sized> IntoIterator for Signatures<E, K> {
    type Item = (E, SigningKeyId<K>, String);
    type IntoIter = IntoIter<E, K>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { outer: self.0.into_iter(), inner: None, entity: None }
    }
}

pub struct IntoIter<E: Clone, K: KeyName + ?Sized> {
    outer: std::collections::btree_map::IntoIter<E, BTreeMap<SigningKeyId<K>, String>>,
    inner: Option<std::collections::btree_map::IntoIter<SigningKeyId<K>, String>>,
    entity: Option<E>,
}

impl<E: Clone, K: KeyName + ?Sized> Iterator for IntoIter<E, K> {
    type Item = (E, SigningKeyId<K>, String);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(inner) = &mut self.inner
                && let Some((k, v)) = inner.next()
                && let Some(entity) = self.entity.clone()
            {
                return Some((entity, k, v));
            }

            if let Some((e, map)) = self.outer.next() {
                self.inner = Some(map.into_iter());
                self.entity = Some(e);
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn signatures_into_iter() {
        use ruma_common::{
            ServerSigningKeyId, Signatures, SigningKeyAlgorithm, owned_server_name,
            owned_server_signing_key_version,
        };
        let key_identifier = ServerSigningKeyId::from_parts(
            SigningKeyAlgorithm::Ed25519,
            &owned_server_signing_key_version!("1"),
        );
        let mut signatures = Signatures::new();
        let server_name = owned_server_name!("example.org");
        let signature = "YbJva03ihSj5mPk+CHMJKUKlCXCPFXjXOK6VqBnN9nA2evksQcTGn6hwQfrgRHIDDXO2le49x7jnWJHMJrJoBQ";
        signatures.insert_signature(server_name, key_identifier, signature.into());

        let mut more_signatures = Signatures::new();
        more_signatures.extend(signatures.clone());

        assert_eq!(more_signatures.0, signatures.0);

        let mut iter = more_signatures.into_iter();
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
    }
}
