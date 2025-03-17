//! Types to deserialize `m.room.third_party_invite` events.

use std::{collections::BTreeSet, ops::Deref};

use ruma_common::{serde::from_raw_json_value, third_party_invite::IdentityServerBase64PublicKey};
use serde::Deserialize;

use super::Event;

/// A helper type for an [`Event`] of type `m.room.third_party_invite`.
///
/// This is a type that deserializes each field lazily, when requested.
#[derive(Debug, Clone)]
pub struct RoomThirdPartyInviteEvent<E: Event>(E);

impl<E: Event> RoomThirdPartyInviteEvent<E> {
    /// Construct a new `RoomThirdPartyInviteEvent` around the given event.
    pub fn new(event: E) -> Self {
        Self(event)
    }

    /// The public keys of the identity server that might be used to sign the third-party invite.
    pub fn public_keys(&self) -> Result<BTreeSet<IdentityServerBase64PublicKey>, String> {
        #[derive(Deserialize)]
        struct RoomThirdPartyInviteContentPublicKeys {
            public_key: Option<IdentityServerBase64PublicKey>,
            #[serde(default)]
            public_keys: Vec<PublicKey>,
        }

        #[derive(Deserialize)]
        struct PublicKey {
            public_key: IdentityServerBase64PublicKey,
        }

        let content: RoomThirdPartyInviteContentPublicKeys = from_raw_json_value(self.content())
            .map_err(|err: serde_json::Error| {
                format!("invalid `public_key` or `public_keys` field in `m.room.third_party_invite` event: {err}")
            })?;
        Ok(content
            .public_key
            .into_iter()
            .chain(content.public_keys.into_iter().map(|k| k.public_key))
            .collect())
    }
}

impl<E: Event> Deref for RoomThirdPartyInviteEvent<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
