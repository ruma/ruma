use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use super::{
    KeyName, OwnedDeviceId, OwnedServerName, OwnedServerSigningKeyVersion, OwnedSigningKeyId,
    OwnedUserId,
};

/// Map of key identifier to signature values.
pub type EntitySignatures<K> = BTreeMap<OwnedSigningKeyId<K>, String>;

/// Map of all signatures, grouped by entity.
///
/// ```
/// # use ruma_common::{server_name, server_signing_key_version, ServerSigningKeyId, Signatures, SigningKeyAlgorithm};
/// let key_identifier = ServerSigningKeyId::from_parts(
///     SigningKeyAlgorithm::Ed25519,
///     server_signing_key_version!("1")
/// );
/// let mut signatures = Signatures::new();
/// let server_name = server_name!("example.org");
/// let signature =
///     "YbJva03ihSj5mPk+CHMJKUKlCXCPFXjXOK6VqBnN9nA2evksQcTGn6hwQfrgRHIDDXO2le49x7jnWJHMJrJoBQ";
/// signatures.insert_signature(server_name, key_identifier, signature.into());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
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
        key_identifier: OwnedSigningKeyId<K>,
        value: String,
    ) -> Option<String> {
        self.0.entry(entity).or_default().insert(key_identifier, value)
    }
}

/// Map of server signatures for an event, grouped by server.
pub type ServerSignatures = Signatures<OwnedServerName, OwnedServerSigningKeyVersion>;

/// Map of device signatures for an event, grouped by user.
pub type DeviceSignatures = Signatures<OwnedUserId, OwnedDeviceId>;

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

impl<E: Ord, K: KeyName + ?Sized, const N: usize> From<[(E, OwnedSigningKeyId<K>, String); N]>
    for Signatures<E, K>
{
    fn from(value: [(E, OwnedSigningKeyId<K>, String); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<E: Ord, K: KeyName + ?Sized> FromIterator<(E, OwnedSigningKeyId<K>, String)>
    for Signatures<E, K>
{
    fn from_iter<T: IntoIterator<Item = (E, OwnedSigningKeyId<K>, String)>>(iter: T) -> Self {
        iter.into_iter().fold(Self::new(), |mut acc, (entity, key_identifier, value)| {
            acc.insert_signature(entity, key_identifier, value);
            acc
        })
    }
}

impl<E: Ord, K: KeyName + ?Sized> Extend<(E, OwnedSigningKeyId<K>, String)> for Signatures<E, K> {
    fn extend<T: IntoIterator<Item = (E, OwnedSigningKeyId<K>, String)>>(&mut self, iter: T) {
        for (entity, key_identifier, value) in iter {
            self.insert_signature(entity, key_identifier, value);
        }
    }
}
