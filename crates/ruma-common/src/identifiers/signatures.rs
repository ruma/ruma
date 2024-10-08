use std::{borrow::Borrow, collections::BTreeMap};

use serde::{Deserialize, Serialize};

use super::{
    KeyName, OwnedDeviceId, OwnedServerName, OwnedServerSigningKeyVersion, OwnedSigningKeyId,
    OwnedUserId,
};

/// Map of key identifier to signature values.
pub type EntitySignatures<K> = BTreeMap<OwnedSigningKeyId<K>, String>;

/// Map of all signatures, grouped by entity
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
/// signatures.insert(server_name, key_identifier, signature.into());
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Signatures<E: Ord, K: KeyName + ?Sized>(BTreeMap<E, EntitySignatures<K>>);

impl<E: Ord, K: KeyName + ?Sized> Signatures<E, K> {
    /// Creates an empty signature map.
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Add a signature for the given server name and key identifier.
    ///
    /// If there was already one, it is returned.
    pub fn insert(
        &mut self,
        entity: E,
        key_identifier: OwnedSigningKeyId<K>,
        value: String,
    ) -> Option<String> {
        self.0.entry(entity).or_default().insert(key_identifier, value)
    }

    /// Returns a reference to the signatures corresponding to the entities.
    pub fn get<Q>(&self, entity: &Q) -> Option<&EntitySignatures<K>>
    where
        E: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.0.get(entity)
    }
}

/// Map of server signatures for an event, grouped by server.
pub type ServerSignatures = Signatures<OwnedServerName, OwnedServerSigningKeyVersion>;

/// Map of device signatures for an event, grouped by user.
pub type DeviceSignatures = Signatures<OwnedUserId, OwnedDeviceId>;
