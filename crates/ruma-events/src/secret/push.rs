//! Types for the [`io.element.msc4385.secret.push`] event.
//!
//! [`io.element.msc4385.secret.push`]: https://github.com/matrix-org/matrix-spec-proposals/pull/4385

use std::fmt;

use ruma_macros::EventContent;
use serde::{Deserialize, Serialize};

use super::request::SecretName;

/// The content of an `m.secret.push` event.
///
/// An event sent by a client to push a secret with another device, without needing an
/// `m.secret.request` event.
///
/// It must be encrypted as an `m.room.encrypted` event, then sent as a to-device event.
#[derive(Clone, Deserialize, Serialize, EventContent)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_event(type = "io.element.msc4385.secret.push", kind = ToDevice)]
pub struct ToDeviceSecretPushEventContent {
    /// The name of the secret.
    pub name: SecretName,

    /// The contents of the secret.
    pub secret: String,
}

impl ToDeviceSecretPushEventContent {
    /// Creates a new `SecretPushEventContent` with the given name and secret.
    pub fn new(name: SecretName, secret: String) -> Self {
        Self { name, secret }
    }
}

impl fmt::Debug for ToDeviceSecretPushEventContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToDeviceSecretPushEventContent")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}
