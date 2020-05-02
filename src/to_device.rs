//! "To-device" versions of encrypted and key verification events.
//!
//! Each "to-device" event includes only the `content`, `type`, and `sender`
//! fields. To-device events are sent directly from one device to the other
//! without the need to create a room.

use ruma_identifiers::UserId;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{
    dummy::DummyEventContent,
    forwarded_room_key::ForwardedRoomKeyEventContent,
    key::verification::{
        accept::AcceptEventContent, cancel::CancelEventContent, key::KeyEventContent,
        mac::MacEventContent, request::RequestEventContent, start::StartEventContent,
    },
    room::encrypted::EncryptedEventContent,
    room_key::RoomKeyEventContent,
    room_key_request::RoomKeyRequestEventContent,
    util::get_field,
    TryFromRaw,
};

/// To-device versions of events that will appear in the to-device part of a
/// sync response.
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum AnyToDeviceEvent {
    /// To-device version of the "m.dummy" event.
    Dummy(ToDeviceDummy),
    /// To-device version of the *m.room_key* event.
    RoomKey(ToDeviceRoomKey),
    /// To-device version of the *m.room.encrypted* event.
    RoomEncrypted(ToDeviceEncrypted),
    /// To-device version of the *m.forwarded_room_key* event.
    ForwardedRoomKey(ToDeviceForwardedRoomKey),
    /// To-device version of the *m.room_key_request* event.
    RoomKeyRequest(ToDeviceRoomKeyRequest),
    /// To-device version of the *m.key.verification.start* event.
    KeyVerificationStart(ToDeviceVerificationStart),
    /// To-device version of the *m.key.verification.accept* event.
    KeyVerificationAccept(ToDeviceVerificationAccept),
    /// To-device version of the *m.key.verification.key* event.
    KeyVerificationKey(ToDeviceVerificationKey),
    /// To-device version of the *m.key.verification.mac* event.
    KeyVerificationMac(ToDeviceVerificationMac),
    /// To-device version of the *m.key.verification.cancel* event.
    KeyVerificationCancel(ToDeviceVerificationCancel),
    /// To-device version of the *m.key.verification.request* event.
    KeyVerificationRequest(ToDeviceVerificationRequest),
}

#[derive(Clone, Debug, Serialize)]
/// To-device event.
pub struct ToDeviceEvent<C> {
    /// The unique identifier for the user who sent this event.
    pub sender: UserId,
    /// Data specific to the event type.
    pub content: C,
}

/// To-device version of the *m.dummy* event.
pub type ToDeviceDummy = ToDeviceEvent<DummyEventContent>;

/// To-device version of the *m.room_key* event.
pub type ToDeviceRoomKey = ToDeviceEvent<RoomKeyEventContent>;

/// To-device version of the *m.room.encrypted* event.
pub type ToDeviceEncrypted = ToDeviceEvent<EncryptedEventContent>;

/// To-device version of the *m.forwarded_room_key* event.
pub type ToDeviceForwardedRoomKey = ToDeviceEvent<ForwardedRoomKeyEventContent>;

/// To-device version of the *m.room_key_request* event.
pub type ToDeviceRoomKeyRequest = ToDeviceEvent<RoomKeyRequestEventContent>;

/// To-device version of the *m.key.verification.start* event.
pub type ToDeviceVerificationStart = ToDeviceEvent<StartEventContent>;

/// To-device version of the *m.key.verification.accept* event.
pub type ToDeviceVerificationAccept = ToDeviceEvent<AcceptEventContent>;

/// To-device version of the *m.key.verification.key* event.
pub type ToDeviceVerificationKey = ToDeviceEvent<KeyEventContent>;

/// To-device version of the *m.key.verification.mac* event.
pub type ToDeviceVerificationMac = ToDeviceEvent<MacEventContent>;

/// To-device version of the *m.key.verification.cancel* event.
pub type ToDeviceVerificationCancel = ToDeviceEvent<CancelEventContent>;

/// To-device version of the *m.key.verification.request* event.
pub type ToDeviceVerificationRequest = ToDeviceEvent<RequestEventContent>;

impl TryFromRaw for AnyToDeviceEvent {
    type Raw = raw::AnyToDeviceEvent;
    type Err = String;

    fn try_from_raw(raw: raw::AnyToDeviceEvent) -> Result<Self, Self::Err> {
        use crate::util::try_convert_variant as conv;
        use raw::AnyToDeviceEvent::*;

        match raw {
            Dummy(c) => conv(AnyToDeviceEvent::Dummy, c),
            RoomKey(c) => conv(AnyToDeviceEvent::RoomKey, c),
            RoomEncrypted(c) => conv(AnyToDeviceEvent::RoomEncrypted, c),
            ForwardedRoomKey(c) => conv(AnyToDeviceEvent::ForwardedRoomKey, c),
            RoomKeyRequest(c) => conv(AnyToDeviceEvent::RoomKeyRequest, c),
            KeyVerificationStart(c) => conv(AnyToDeviceEvent::KeyVerificationStart, c),
            KeyVerificationAccept(c) => conv(AnyToDeviceEvent::KeyVerificationAccept, c),
            KeyVerificationKey(c) => conv(AnyToDeviceEvent::KeyVerificationKey, c),
            KeyVerificationMac(c) => conv(AnyToDeviceEvent::KeyVerificationMac, c),
            KeyVerificationCancel(c) => conv(AnyToDeviceEvent::KeyVerificationCancel, c),
            KeyVerificationRequest(c) => conv(AnyToDeviceEvent::KeyVerificationRequest, c),
        }
    }
}

impl<C> TryFromRaw for ToDeviceEvent<C>
where
    C: TryFromRaw,
{
    type Raw = ToDeviceEvent<C::Raw>;
    type Err = C::Err;

    fn try_from_raw(raw: ToDeviceEvent<C::Raw>) -> Result<Self, Self::Err> {
        Ok(Self {
            content: C::try_from_raw(raw.content)?,
            sender: raw.sender,
        })
    }
}

impl<'de, C> Deserialize<'de> for ToDeviceEvent<C>
where
    C: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: Optimize, what should be optimized here? Can we expand this
        // comment?
        let value = Value::deserialize(deserializer)?;

        Ok(Self {
            content: get_field(&value, "content")?,
            sender: get_field(&value, "sender")?,
        })
    }
}

mod raw {
    use serde::{Deserialize, Deserializer};
    use serde_json::Value;

    use super::ToDeviceEvent;
    use crate::{
        dummy::DummyEventContent,
        forwarded_room_key::raw::ForwardedRoomKeyEventContent,
        key::verification::{
            accept::raw::AcceptEventContent, cancel::raw::CancelEventContent,
            key::raw::KeyEventContent, mac::raw::MacEventContent,
            request::raw::RequestEventContent, start::raw::StartEventContent,
        },
        room::encrypted::raw::EncryptedEventContent,
        room_key::raw::RoomKeyEventContent,
        room_key_request::raw::RoomKeyRequestEventContent,
        util::get_field,
    };

    /// To-device version of the *m.dummy* event.
    pub type ToDeviceDummy = ToDeviceEvent<DummyEventContent>;
    /// To-device version of the *m.room_key* event.
    pub type ToDeviceRoomKey = ToDeviceEvent<RoomKeyEventContent>;
    /// To-device version of the *m.room.encrypted* event.
    pub type ToDeviceEncrypted = ToDeviceEvent<EncryptedEventContent>;
    /// To-device version of the *m.forwarded_room_key* event.
    pub type ToDeviceForwardedRoomKey = ToDeviceEvent<ForwardedRoomKeyEventContent>;
    /// To-device version of the *m.room_key_request* event.
    pub type ToDeviceRoomKeyRequest = ToDeviceEvent<RoomKeyRequestEventContent>;
    /// To-device version of the *m.key.verification.start* event.
    pub type ToDeviceVerificationStart = ToDeviceEvent<StartEventContent>;
    /// To-device version of the *m.key.verification.accept* event.
    pub type ToDeviceVerificationAccept = ToDeviceEvent<AcceptEventContent>;
    /// To-device version of the *m.key.verification.key* event.
    pub type ToDeviceVerificationKey = ToDeviceEvent<KeyEventContent>;
    /// To-device version of the *m.key.verification.mac* event.
    pub type ToDeviceVerificationMac = ToDeviceEvent<MacEventContent>;
    /// To-device version of the *m.key.verification.cancel* event.
    pub type ToDeviceVerificationCancel = ToDeviceEvent<CancelEventContent>;
    /// To-device version of the *m.key.verification.request* event.
    pub type ToDeviceVerificationRequest = ToDeviceEvent<RequestEventContent>;

    /// A stripped-down version of a state event that is included along with some other events.
    #[derive(Clone, Debug)]
    #[allow(clippy::large_enum_variant)]
    pub enum AnyToDeviceEvent {
        /// To-device version of the "m.dummy" event.
        Dummy(ToDeviceDummy),
        /// To-device version of the *m.room_key* event.
        RoomKey(ToDeviceRoomKey),
        /// To-device version of the *m.room.encrypted* event.
        RoomEncrypted(ToDeviceEncrypted),
        /// To-device version of the *m.forwarded_room_key* event.
        ForwardedRoomKey(ToDeviceForwardedRoomKey),
        /// To-device version of the *m.room_key_request* event.
        RoomKeyRequest(ToDeviceRoomKeyRequest),
        /// To-device version of the *m.key.verification.start* event.
        KeyVerificationStart(ToDeviceVerificationStart),
        /// To-device version of the *m.key.verification.accept* event.
        KeyVerificationAccept(ToDeviceVerificationAccept),
        /// To-device version of the *m.key.verification.key* event.
        KeyVerificationKey(ToDeviceVerificationKey),
        /// To-device version of the *m.key.verification.mac* event.
        KeyVerificationMac(ToDeviceVerificationMac),
        /// To-device version of the *m.key.verification.cancel* event.
        KeyVerificationCancel(ToDeviceVerificationCancel),
        /// To-device version of the *m.key.verification.request* event.
        KeyVerificationRequest(ToDeviceVerificationRequest),
    }

    impl<'de> Deserialize<'de> for AnyToDeviceEvent {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            use crate::{util::try_variant_from_value as from_value, EventType::*};
            use serde::de::Error as _;

            // TODO: Optimize, what should be optimized here? Can we expand this
            // comment?
            let value = Value::deserialize(deserializer)?;
            let event_type = get_field(&value, "type")?;

            match event_type {
                Dummy => from_value(value, AnyToDeviceEvent::Dummy),
                RoomKey => from_value(value, AnyToDeviceEvent::RoomKey),
                RoomEncrypted => from_value(value, AnyToDeviceEvent::RoomEncrypted),
                ForwardedRoomKey => from_value(value, AnyToDeviceEvent::ForwardedRoomKey),
                RoomKeyRequest => from_value(value, AnyToDeviceEvent::RoomKeyRequest),
                KeyVerificationStart => from_value(value, AnyToDeviceEvent::KeyVerificationStart),
                KeyVerificationAccept => from_value(value, AnyToDeviceEvent::KeyVerificationAccept),
                KeyVerificationKey => from_value(value, AnyToDeviceEvent::KeyVerificationKey),
                KeyVerificationMac => from_value(value, AnyToDeviceEvent::KeyVerificationMac),
                KeyVerificationCancel => from_value(value, AnyToDeviceEvent::KeyVerificationCancel),
                KeyVerificationRequest => {
                    from_value(value, AnyToDeviceEvent::KeyVerificationRequest)
                }
                _ => Err(D::Error::custom("unknown to-device event")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        convert::TryFrom,
        time::{Duration, UNIX_EPOCH},
    };

    use ruma_identifiers::{RoomId, UserId};
    use serde_json::{from_value as from_json_value, json};

    use super::AnyToDeviceEvent;
    use crate::{
        key::verification::{
            cancel::CancelCode, start::StartEventContent, HashAlgorithm, KeyAgreementProtocol,
            MessageAuthenticationCode, ShortAuthenticationString, VerificationMethod,
        },
        room::encrypted::EncryptedEventContent,
        room_key_request::Action,
        Algorithm, Empty, EventJson,
    };

    macro_rules! deserialize {
        ($source:ident, $($target:tt)*) => {{
            let event = from_json_value::<EventJson<AnyToDeviceEvent>>($source)
                .expect(&format!(
                    "Can't deserialize to-device event: {} from source {}",
                    stringify!($($target)*), stringify!($source)
                ));

            let event = event
                .deserialize()
                .expect("To-device event {} deserialized into a invalid event");

            match event {
                $($target)*(e) => {
                    assert_eq!(
                        e.sender,
                        UserId::try_from("@alice:example.org").unwrap()
                    );
                    e
                },
                _ => panic!(
                    "{} event deserialized into a incorrect event type",
                    stringify!($($target)*)
                ),
            }
        }};
    }

    #[test]
    fn dummy() {
        let dummy = json!({
            "content": {},
            "sender": "@alice:example.org",
            "type": "m.dummy"
        });

        let event = deserialize! {dummy, AnyToDeviceEvent::Dummy};

        assert_eq!(event.content, Empty);
    }

    #[test]
    fn room_key() {
        let room_key = json!({
            "content": {
                "algorithm": "m.megolm.v1.aes-sha2",
                "room_id": "!test:localhost",
                "session_id": "fake_id",
                "session_key": "fake_key"
            },
            "sender": "@alice:example.org",
            "type": "m.room_key"
        });

        let event = deserialize! {room_key, AnyToDeviceEvent::RoomKey};

        assert_eq!(
            event.content.room_id,
            RoomId::try_from("!test:localhost").unwrap()
        );
        assert_eq!(event.content.session_id, "fake_id");
        assert_eq!(event.content.session_key, "fake_key");
        assert_eq!(event.content.algorithm, Algorithm::MegolmV1AesSha2);
    }

    #[test]
    fn encrypted_olm() {
        let source = json!({
            "content": {
                "sender_key": "test_sender_key",
                "ciphertext": {
                    "sender_key_0": {
                        "body": "ciphertext0",
                        "type": 0
                    },
                    "sender_key_1": {
                        "body": "ciphertext1",
                        "type": 1
                    }
                },
                "algorithm": "m.olm.v1.curve25519-aes-sha2"
            },
            "type": "m.room.encrypted",
            "sender": "@alice:example.org"
        });

        let event = deserialize! {source, AnyToDeviceEvent::RoomEncrypted};

        let content = match &event.content {
            EncryptedEventContent::OlmV1Curve25519AesSha2(c) => c,
            _ => panic!("Wrong content type, expected a OlmV1 content"),
        };

        assert_eq!(content.algorithm, Algorithm::OlmV1Curve25519AesSha2);
        assert_eq!(content.sender_key, "test_sender_key");
        assert_eq!(content.ciphertext.len(), 2);
        assert_eq!(content.ciphertext["sender_key_0"].body, "ciphertext0");
        assert_eq!(content.ciphertext["sender_key_0"].message_type, 0u16.into());
        assert_eq!(content.ciphertext["sender_key_1"].body, "ciphertext1");
        assert_eq!(content.ciphertext["sender_key_1"].message_type, 1u16.into());
    }

    #[test]
    fn forwarded_room_key() {
        let source = json!({
            "content": {
                "algorithm": "m.megolm.v1.aes-sha2",
                "forwarding_curve25519_key_chain": [
                    "hPQNcabIABgGnx3/ACv/jmMmiQHoeFfuLB17tzWp6Hw"
                ],
                "room_id": "!test:localhost",
                "sender_claimed_ed25519_key": "aj40p+aw64yPIdsxoog8jhPu9i7l7NcFRecuOQblE3Y",
                "sender_key": "RF3s+E7RkTQTGF2d8Deol0FkQvgII2aJDf3/Jp5mxVU",
                "session_id": "fake_id",
                "session_key": "fake_key"
            },
            "sender": "@alice:example.org",
            "type": "m.forwarded_room_key"
        });

        let event = deserialize! {source, AnyToDeviceEvent::ForwardedRoomKey};

        assert_eq!(
            event.content.room_id,
            RoomId::try_from("!test:localhost").unwrap()
        );
        assert_eq!(event.content.session_id, "fake_id");
        assert_eq!(event.content.session_key, "fake_key");
        assert_eq!(event.content.algorithm, Algorithm::MegolmV1AesSha2);
        assert_eq!(
            event.content.forwarding_curve25519_key_chain,
            ["hPQNcabIABgGnx3/ACv/jmMmiQHoeFfuLB17tzWp6Hw"]
        );
        assert_eq!(
            event.content.sender_claimed_ed25519_key,
            "aj40p+aw64yPIdsxoog8jhPu9i7l7NcFRecuOQblE3Y"
        );
    }

    #[test]
    fn key_request() {
        let source = json!({
            "sender": "@alice:example.org",
            "content": {
                "action": "request",
                "body": {
                    "algorithm": "m.megolm.v1.aes-sha2",
                    "room_id": "!test:localhost",
                    "sender_key": "RF3s+E7RkTQTGF2d8Deol0FkQvgII2aJDf3/Jp5mxVU",
                    "session_id": "X3lUlvLELLYxeTx4yOVu6UDpasGEVO0Jbu+QFnm0cKQ"
                },
                "request_id": "1495474790150.19",
                "requesting_device_id": "RJYKSTBOIE"
            },
            "type": "m.room_key_request"
        });

        let event = deserialize! {source, AnyToDeviceEvent::RoomKeyRequest};
        let body = event.content.body.as_ref().unwrap();

        assert_eq!(event.content.action, Action::Request);
        assert_eq!(event.content.request_id, "1495474790150.19");
        assert_eq!(event.content.requesting_device_id, "RJYKSTBOIE");
        assert_eq!(body.room_id, RoomId::try_from("!test:localhost").unwrap());
        assert_eq!(
            body.sender_key,
            "RF3s+E7RkTQTGF2d8Deol0FkQvgII2aJDf3/Jp5mxVU"
        );
        assert_eq!(
            body.session_id,
            "X3lUlvLELLYxeTx4yOVu6UDpasGEVO0Jbu+QFnm0cKQ"
        );
    }

    #[test]
    fn key_request_cancel() {
        let source = json!({
            "sender": "@alice:example.org",
            "content": {
                "action": "request_cancellation",
                "request_id": "1495474790150.19",
                "requesting_device_id": "RJYKSTBOIE"
            },
            "type": "m.room_key_request"
        });

        let event = deserialize! {source, AnyToDeviceEvent::RoomKeyRequest};
        assert_eq!(event.content.action, Action::CancelRequest);
        assert_eq!(event.content.request_id, "1495474790150.19");
        assert_eq!(event.content.requesting_device_id, "RJYKSTBOIE");
    }

    #[test]
    fn key_verification_start() {
        let source = json!({
            "content": {
                "from_device": "AliceDevice1",
                "hashes": [
                    "sha256"
                ],
                "key_agreement_protocols": [
                    "curve25519"
                ],
                "message_authentication_codes": [
                    "hkdf-hmac-sha256"
                ],
                "method": "m.sas.v1",
                "short_authentication_string": [
                    "decimal",
                    "emoji"
                ],
                "transaction_id": "S0meUniqueAndOpaqueString"
            },
            "type": "m.key.verification.start",
            "sender": "@alice:example.org"
        });

        let event = deserialize! {source, AnyToDeviceEvent::KeyVerificationStart};

        let content = match &event.content {
            StartEventContent::MSasV1(c) => c,
            _ => panic!("Key verification content deserialized into the wrong content type"),
        };

        assert_eq!(content.from_device, "AliceDevice1");
        assert_eq!(content.hashes, &[HashAlgorithm::Sha256]);
        assert_eq!(
            content.key_agreement_protocols,
            &[KeyAgreementProtocol::Curve25519]
        );
        assert_eq!(
            content.message_authentication_codes,
            &[MessageAuthenticationCode::HkdfHmacSha256]
        );
        assert_eq!(
            content.short_authentication_string,
            &[
                ShortAuthenticationString::Decimal,
                ShortAuthenticationString::Emoji
            ]
        );
        assert_eq!(content.transaction_id, "S0meUniqueAndOpaqueString");
    }

    #[test]
    fn key_verification_accept() {
        let source = json!({
            "content": {
                "commitment": "fQpGIW1Snz+pwLZu6sTy2aHy/DYWWTspTJRPyNp0PKkymfIsNffysMl6ObMMFdIJhk6g6pwlIqZ54rxo8SLmAg",
                "hash": "sha256",
                "key_agreement_protocol": "curve25519",
                "message_authentication_code": "hkdf-hmac-sha256",
                "method": "m.sas.v1",
                "short_authentication_string": [
                    "decimal",
                    "emoji"
                ],
                "transaction_id": "S0meUniqueAndOpaqueString"
            },
            "type": "m.key.verification.accept",
            "sender": "@alice:example.org"
        });

        let event = deserialize! {source, AnyToDeviceEvent::KeyVerificationAccept};
        assert_eq!(event.content.hash, HashAlgorithm::Sha256);
        assert_eq!(
            event.content.commitment,
            "fQpGIW1Snz+pwLZu6sTy2aHy/DYWWTspTJRPyNp0PKkymfIsNffysMl6ObMMFdIJhk6g6pwlIqZ54rxo8SLmAg"
        );
        assert_eq!(
            event.content.key_agreement_protocol,
            KeyAgreementProtocol::Curve25519
        );
        assert_eq!(
            event.content.message_authentication_code,
            MessageAuthenticationCode::HkdfHmacSha256
        );
        assert_eq!(
            event.content.short_authentication_string,
            &[
                ShortAuthenticationString::Decimal,
                ShortAuthenticationString::Emoji
            ]
        );
        assert_eq!(event.content.method, VerificationMethod::MSasV1);
        assert_eq!(event.content.transaction_id, "S0meUniqueAndOpaqueString");
    }

    #[test]
    fn key_verification_key() {
        let source = json!({
            "content": {
                "key": "fQpGIW1Snz+pwLZu6sTy2aHy/DYWWTspTJRPyNp0PKkymfIsNffysMl6ObMMFdIJhk6g6pwlIqZ54rxo8SLmAg",
                "transaction_id": "S0meUniqueAndOpaqueString"
            },
            "type": "m.key.verification.key",
            "sender": "@alice:example.org"
        });

        let event = deserialize! {source, AnyToDeviceEvent::KeyVerificationKey};

        assert_eq!(event.content.transaction_id, "S0meUniqueAndOpaqueString");
        assert_eq!(
            event.content.key,
            "fQpGIW1Snz+pwLZu6sTy2aHy/DYWWTspTJRPyNp0PKkymfIsNffysMl6ObMMFdIJhk6g6pwlIqZ54rxo8SLmAg"
        );
    }

    #[test]
    fn key_verification_mac() {
        let source = json!({
            "content": {
                "keys": "2Wptgo4CwmLo/Y8B8qinxApKaCkBG2fjTWB7AbP5Uy+aIbygsSdLOFzvdDjww8zUVKCmI02eP9xtyJxc/cLiBA",
                "mac": {
                    "ed25519:ABCDEF": "fQpGIW1Snz+pwLZu6sTy2aHy/DYWWTspTJRPyNp0PKkymfIsNffysMl6ObMMFdIJhk6g6pwlIqZ54rxo8SLmAg"
                },
                "transaction_id": "S0meUniqueAndOpaqueString"
            },
            "type": "m.key.verification.mac",
            "sender": "@alice:example.org"
        });

        let event = deserialize! {source, AnyToDeviceEvent::KeyVerificationMac};
        assert_eq!(event.content.transaction_id, "S0meUniqueAndOpaqueString");
        assert_eq!(
            event.content.keys,
            "2Wptgo4CwmLo/Y8B8qinxApKaCkBG2fjTWB7AbP5Uy+aIbygsSdLOFzvdDjww8zUVKCmI02eP9xtyJxc/cLiBA"
        );
        assert_eq!(
            event.content.mac["ed25519:ABCDEF"],
            "fQpGIW1Snz+pwLZu6sTy2aHy/DYWWTspTJRPyNp0PKkymfIsNffysMl6ObMMFdIJhk6g6pwlIqZ54rxo8SLmAg"
        );
    }

    #[test]
    fn key_verification_cancel() {
        let source = json!({
            "content": {
                "code": "m.user",
                "reason": "Some reason",
                "transaction_id": "S0meUniqueAndOpaqueString"
            },
            "type": "m.key.verification.cancel",
            "sender": "@alice:example.org"
        });

        let event = deserialize! {source, AnyToDeviceEvent::KeyVerificationCancel};
        assert_eq!(event.content.transaction_id, "S0meUniqueAndOpaqueString");
        assert_eq!(event.content.reason, "Some reason");
        assert_eq!(event.content.code, CancelCode::User);
    }

    #[test]
    fn key_verification_request() {
        let source = json!({
            "content": {
                "from_device": "AliceDevice2",
                "methods": [
                    "m.sas.v1"
                ],
                "timestamp": 1_559_598_944_869_u64,
                "transaction_id": "S0meUniqueAndOpaqueString"
            },
            "type": "m.key.verification.request",
            "sender": "@alice:example.org"
        });

        let event = deserialize! {source, AnyToDeviceEvent::KeyVerificationRequest};
        assert_eq!(event.content.transaction_id, "S0meUniqueAndOpaqueString");
        assert_eq!(event.content.from_device, "AliceDevice2");
        assert_eq!(event.content.methods, &[VerificationMethod::MSasV1]);
        assert_eq!(
            event.content.timestamp,
            UNIX_EPOCH + Duration::from_millis(1_559_598_944_869)
        );
    }
}
