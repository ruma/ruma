//! Types for [Matrix](https://matrix.org/) identifiers for devices, events, keys, rooms, servers,
//! users and URIs.

// FIXME: Remove once lint doesn't trigger on std::convert::TryFrom in identifiers/macros.rs anymore
#![allow(unused_qualifications)]

#[doc(inline)]
pub use ruma_identifiers_validation::{
    ID_MAX_BYTES, KeyName,
    error::{
        Error as IdParseError, MatrixIdError, MatrixToError, MatrixUriError, MxcUriError,
        VoipVersionIdError,
    },
};
use serde::de::{self, Deserializer, Unexpected};

#[doc(inline)]
pub use self::{
    base64_public_key::Base64PublicKey,
    base64_public_key_or_device_id::Base64PublicKeyOrDeviceId,
    client_secret::ClientSecret,
    crypto_algorithms::{
        DeviceKeyAlgorithm, EventEncryptionAlgorithm, KeyDerivationAlgorithm, OneTimeKeyAlgorithm,
        SigningKeyAlgorithm,
    },
    device_id::DeviceId,
    direct_user_identifier::{DirectUserIdentifier, OwnedDirectUserIdentifier},
    event_id::EventId,
    key_id::{
        AnyKeyName, CrossSigningKeyId, CrossSigningOrDeviceSigningKeyId, DeviceKeyId,
        DeviceSigningKeyId, KeyAlgorithm, KeyId, OneTimeKeyId, ServerSigningKeyId, SigningKeyId,
    },
    matrix_uri::{MatrixToUri, MatrixUri},
    mxc_uri::MxcUri,
    one_time_key_name::OneTimeKeyName,
    room_alias_id::RoomAliasId,
    room_id::RoomId,
    room_or_alias_id::RoomOrAliasId,
    room_version_id::RoomVersionId,
    server_name::ServerName,
    server_signing_key_version::ServerSigningKeyVersion,
    session_id::SessionId,
    signatures::{
        CrossSigningOrDeviceSignatures, DeviceSignatures, EntitySignatures, ServerSignatures,
        Signatures,
    },
    space_child_order::SpaceChildOrder,
    transaction_id::TransactionId,
    user_id::UserId,
    voip_id::{OwnedVoipId, VoipId},
    voip_version_id::VoipVersionId,
};

pub mod matrix_uri;
pub mod user_id;

mod base64_public_key;
mod base64_public_key_or_device_id;
mod client_secret;
mod crypto_algorithms;
mod device_id;
mod direct_user_identifier;
mod event_id;
mod key_id;
mod mxc_uri;
mod one_time_key_name;
mod room_alias_id;
mod room_id;
mod room_or_alias_id;
mod room_version_id;
mod server_name;
mod server_signing_key_version;
mod session_id;
mod signatures;
mod space_child_order;
mod transaction_id;
mod voip_id;
mod voip_version_id;

/// Generates a random identifier localpart.
#[cfg(feature = "rand")]
fn generate_localpart(length: usize) -> Box<str> {
    use rand::RngExt as _;
    rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .map(char::from)
        .take(length)
        .collect::<String>()
        .into_boxed_str()
}

/// Find the localpart in the given identifier string.
///
/// This function expects the string to start with a sigil and the localpart to be the part between
/// the sigil and the first colon. If there is no colon, the full string after the sigil is assumed
/// to be the localpart.
fn find_localpart(s: &str) -> &str {
    let without_sigil = &s[1..];
    without_sigil.find(':').map(|idx| &without_sigil[..idx]).unwrap_or(without_sigil)
}

/// Find the server name in the given identifier string and return it as a `&str`.
///
/// This function expects the server name to be the part of the string after the first colon.
///
/// Returns `None` if there is no colon in the string.
fn find_server_name_str(s: &str) -> Option<&str> {
    s.find(':').map(|idx| &s[idx + 1..])
}

/// Find the server name from the given identifier string an return it as a `ServerName`.
///
/// This function expects the server name to be the part of the string after the first colon, and
/// that it was already validated.
///
/// Returns `None` if there is no colon in the string.
fn find_server_name_unchecked(s: &str) -> Option<ServerName> {
    find_server_name_str(s).map(ServerName::from_str_unchecked)
}

/// Deserializes any type of id using the provided `TryFrom` implementation.
///
/// This is a helper function to reduce the boilerplate of the `Deserialize` implementations.
fn deserialize_id<'de, D, T>(deserializer: D, expected_str: &str) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: for<'a> TryFrom<&'a str>,
{
    crate::serde::deserialize_cow_str(deserializer).and_then(|v| {
        T::try_from(&v).map_err(|_| de::Error::invalid_value(Unexpected::Str(&v), &expected_str))
    })
}

/// Shorthand for `DeviceId::from`.
#[macro_export]
macro_rules! device_id {
    ($s:expr) => {
        <$crate::DeviceId as ::std::convert::From<_>>::from($s)
    };
}

/// `&'static DeviceId` construction.
///
/// This macro is a helper to ease the transition after the change of [`DeviceId`] from a
/// dynamically sized type to an owned type. It has the side effect of interning and leaking the
/// identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! device_id_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::DEVICE_ID_INTERNER
            .get_or_insert_with($s, || $crate::device_id!($s))
    };
}

#[doc(hidden)]
pub mod __private_macros {
    pub use ruma_macros::{
        base64_public_key, event_id, mxc_uri, room_alias_id, room_id, room_version_id, server_name,
        server_signing_key_version, session_id, user_id,
    };

    #[cfg(feature = "unstable-identifier-ref-macros")]
    pub mod id_interner {
        use std::sync::LazyLock;

        /// Type used to intern identifiers, used in macros to return a static reference.
        #[doc(hidden)]
        pub struct IdInterner<T: 'static> {
            // Map of static string for the identifier to identifier.
            #[allow(clippy::disallowed_types)]
            inner: std::sync::RwLock<std::collections::HashMap<&'static str, &'static T>>,
        }

        impl<T: 'static> IdInterner<T> {
            /// Construct an empty `IdInterner`.
            fn new() -> Self {
                Self { inner: Default::default() }
            }

            /// Get the identifier matching the given key or create it with the given function.
            pub fn get_or_insert_with<F>(&self, key: &'static str, f: F) -> &'static T
            where
                F: FnOnce() -> T,
            {
                // First, acquire a read lock to check if the identifier exists in the map.
                if let Some(id) = self.inner.read().expect("lock should never be poisoned").get(key)
                {
                    return id;
                }

                // It is not in the map, acquire a write lock to add it.
                self.inner
                    .write()
                    .expect("lock should never be poisoned")
                    .entry(key)
                    .or_insert_with(|| {
                        let id = f();
                        Box::leak(Box::new(id))
                    })
            }
        }

        pub static BASE64_PUBLIC_KEY_INTERNER: LazyLock<IdInterner<crate::Base64PublicKey>> =
            LazyLock::new(IdInterner::new);

        pub static DEVICE_ID_INTERNER: LazyLock<IdInterner<crate::DeviceId>> =
            LazyLock::new(IdInterner::new);

        pub static EVENT_ID_INTERNER: LazyLock<IdInterner<crate::EventId>> =
            LazyLock::new(IdInterner::new);

        pub static MXC_URI_INTERNER: LazyLock<IdInterner<crate::MxcUri>> =
            LazyLock::new(IdInterner::new);

        pub static ROOM_ALIAS_ID_INTERNER: LazyLock<IdInterner<crate::RoomAliasId>> =
            LazyLock::new(IdInterner::new);

        pub static ROOM_ID_INTERNER: LazyLock<IdInterner<crate::RoomId>> =
            LazyLock::new(IdInterner::new);

        pub static SERVER_NAME_INTERNER: LazyLock<IdInterner<crate::ServerName>> =
            LazyLock::new(IdInterner::new);

        pub static SERVER_SIGNING_KEY_VERSION_INTERNER: LazyLock<
            IdInterner<crate::ServerSigningKeyVersion>,
        > = LazyLock::new(IdInterner::new);

        pub static SESSION_ID_INTERNER: LazyLock<IdInterner<crate::SessionId>> =
            LazyLock::new(IdInterner::new);

        pub static USER_ID_INTERNER: LazyLock<IdInterner<crate::UserId>> =
            LazyLock::new(IdInterner::new);
    }
}

/// Compile-time checked [`EventId`] construction.
#[macro_export]
macro_rules! event_id {
    ($s:literal) => {
        $crate::__private_macros::event_id!($crate, $s)
    };
}

/// Compile-time checked `&'static EventId` construction.
///
/// This macro is a helper to ease the transition after the change of [`EventId`] from a
/// dynamically sized type to an owned type. It has the side effect of interning and leaking the
/// identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! event_id_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::EVENT_ID_INTERNER
            .get_or_insert_with($s, || $crate::event_id!($s))
    };
}

/// Compile-time checked [`RoomAliasId`] construction.
#[macro_export]
macro_rules! room_alias_id {
    ($s:literal) => {
        $crate::__private_macros::room_alias_id!($crate, $s)
    };
}

/// Compile-time checked `&'static RoomAliasId` construction.
///
/// This macro is a helper to ease the transition after the change of [`RoomAliasId`] from a
/// dynamically sized type to an owned type. It has the side effect of interning and leaking the
/// identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! room_alias_id_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::ROOM_ALIAS_ID_INTERNER
            .get_or_insert_with($s, || $crate::room_alias_id!($s))
    };
}

/// Compile-time checked [`RoomId`] construction.
#[macro_export]
macro_rules! room_id {
    ($s:literal) => {
        $crate::__private_macros::room_id!($crate, $s)
    };
}

/// Compile-time checked `&'static RoomId` construction.
///
/// This macro is a helper to ease the transition after the change of [`RoomId`] from a
/// dynamically sized type to an owned type. It has the side effect of interning and leaking the
/// identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! room_id_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::ROOM_ID_INTERNER
            .get_or_insert_with($s, || $crate::room_id!($s))
    };
}

/// Compile-time checked [`RoomVersionId`] construction.
#[macro_export]
macro_rules! room_version_id {
    ($s:literal) => {
        $crate::__private_macros::room_version_id!($crate, $s)
    };
}

/// Compile-time checked [`ServerSigningKeyVersion`] construction.
#[macro_export]
macro_rules! server_signing_key_version {
    ($s:literal) => {
        $crate::__private_macros::server_signing_key_version!($crate, $s)
    };
}

/// Compile-time checked `&'static ServerSigningKeyVersion` construction.
///
/// This macro is a helper to ease the transition after the change of [`ServerSigningKeyVersion`]
/// from a dynamically sized type to an owned type. It has the side effect of interning and leaking
/// the identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! server_signing_key_version_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::SERVER_SIGNING_KEY_VERSION_INTERNER
            .get_or_insert_with($s, || $crate::server_signing_key_version!($s))
    };
}

/// Compile-time checked [`ServerName`] construction.
#[macro_export]
macro_rules! server_name {
    ($s:literal) => {
        $crate::__private_macros::server_name!($crate, $s)
    };
}

/// Compile-time checked `&'static ServerName` construction.
///
/// This macro is a helper to ease the transition after the change of [`ServerName`] from a
/// dynamically sized type to an owned type. It has the side effect of interning and leaking the
/// identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! server_name_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::SERVER_NAME_INTERNER
            .get_or_insert_with($s, || $crate::server_name!($s))
    };
}

/// Compile-time checked [`SessionId`] construction.
#[macro_export]
macro_rules! session_id {
    ($s:literal) => {{ $crate::__private_macros::session_id!($crate, $s) }};
}

/// Compile-time checked `&'static SessionId` construction.
///
/// This macro is a helper to ease the transition after the change of [`SessionId`] from a
/// dynamically sized type to an owned type. It has the side effect of interning and leaking the
/// identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! session_id_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::SESSION_ID_INTERNER
            .get_or_insert_with($s, || $crate::session_id!($s))
    };
}

/// Compile-time checked [`MxcUri`] construction.
#[macro_export]
macro_rules! mxc_uri {
    ($s:literal) => {
        $crate::__private_macros::mxc_uri!($crate, $s)
    };
}

/// Compile-time checked `&'static MxcUri` construction.
///
/// This macro is a helper to ease the transition after the change of [`MxcUri`] from a
/// dynamically sized type to an owned type. It has the side effect of interning and leaking the
/// identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! mxc_uri_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::MXC_URI_INTERNER
            .get_or_insert_with($s, || $crate::mxc_uri!($s))
    };
}

/// Compile-time checked [`UserId`] construction.
#[macro_export]
macro_rules! user_id {
    ($s:literal) => {
        $crate::__private_macros::user_id!($crate, $s)
    };
}

/// Compile-time checked `&'static UserId` construction.
///
/// This macro is a helper to ease the transition after the change of [`UserId`] from a
/// dynamically sized type to an owned type. It has the side effect of interning and leaking the
/// identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! user_id_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::USER_ID_INTERNER
            .get_or_insert_with($s, || $crate::user_id!($s))
    };
}

/// Compile-time checked [`Base64PublicKey`] construction.
#[macro_export]
macro_rules! base64_public_key {
    ($s:literal) => {
        $crate::__private_macros::base64_public_key!($crate, $s)
    };
}

/// Compile-time checked `&'static Base64PublicKey` construction.
///
/// This macro is a helper to ease the transition after the change of [`Base64PublicKey`] from a
/// dynamically sized type to an owned type. It has the side effect of interning and leaking the
/// identifier so it SHOULD NOT be used in code that runs in production.
///
/// This is behind the `unstable-identifier-ref-macros` cargo feature to allow us to remove this
/// macro at any time without it being a breaking change.
#[macro_export]
#[cfg(feature = "unstable-identifier-ref-macros")]
macro_rules! base64_public_key_ref {
    ($s:literal) => {
        $crate::__private_macros::id_interner::BASE64_PUBLIC_KEY_INTERNER
            .get_or_insert_with($s, || $crate::base64_public_key!($s))
    };
}
