use std::borrow::Cow;

use ruma_common::serde::Base64;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use super::V2EncryptedFileInfo;

impl<'de> Deserialize<'de> for V2EncryptedFileInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let V2EncryptedFileInfoSerdeHelper { key: JsonWebKey { kty, key_ops, alg, k, ext }, iv } =
            V2EncryptedFileInfoSerdeHelper::deserialize(deserializer)?;

        if kty != "oct" {
            return Err(de::Error::custom(format!(
                "invalid value in `kty` field: `{kty}` , expected `oct`"
            )));
        }

        if alg != "A256CTR" {
            return Err(de::Error::custom(format!(
                "invalid value in `alg` field: `{alg}` , expected `A256CTR`"
            )));
        }

        if !key_ops.iter().any(|key_op| key_op == "encrypt") {
            return Err(de::Error::custom("missing value `encrypt` in `key_ops` field"));
        }

        if !key_ops.iter().any(|key_op| key_op == "decrypt") {
            return Err(de::Error::custom("missing value `decrypt` in `key_ops` field"));
        }

        if !ext {
            return Err(de::Error::custom(
                "invalid value in `ext` field: `false` , expected `true`",
            ));
        }

        let k = Base64::parse(k.as_ref())
            .map_err(|error| de::Error::custom(format!("invalid value in `k` field: {error}")))?;
        let iv = Base64::parse(iv.as_ref())
            .map_err(|error| de::Error::custom(format!("invalid value in `iv` field: {error}")))?;

        Ok(Self { k, iv })
    }
}

impl Serialize for V2EncryptedFileInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Self { k, iv } = self;

        let info = V2EncryptedFileInfoSerdeHelper {
            key: JsonWebKey {
                kty: Cow::Borrowed("oct"),
                key_ops: vec![Cow::Borrowed("decrypt"), Cow::Borrowed("encrypt")],
                alg: Cow::Borrowed("A256CTR"),
                k: Cow::Owned(k.encode()),
                ext: true,
            },
            iv: Cow::Owned(iv.encode()),
        };

        info.serialize(serializer)
    }
}

#[derive(Deserialize, Serialize)]
struct V2EncryptedFileInfoSerdeHelper<'a> {
    /// The key.
    #[serde(borrow)]
    key: JsonWebKey<'a>,

    /// The 128-bit unique counter block used by AES-CTR, encoded as unpadded base64.
    #[serde(borrow)]
    iv: Cow<'a, str>,
}

/// A [JSON Web Key](https://tools.ietf.org/html/rfc7517#appendix-A.3) object.
#[derive(Deserialize, Serialize)]
struct JsonWebKey<'a> {
    /// Key type.
    ///
    /// Must be `oct`.
    #[serde(borrow)]
    kty: Cow<'a, str>,

    /// Key operations.
    ///
    /// Must at least contain `encrypt` and `decrypt`.
    #[serde(borrow)]
    key_ops: Vec<Cow<'a, str>>,

    /// Algorithm.
    ///
    /// Must be `A256CTR`.
    #[serde(borrow)]
    alg: Cow<'a, str>,

    /// The key, encoded as url-safe unpadded base64.
    #[serde(borrow)]
    k: Cow<'a, str>,

    /// Extractable.
    ///
    /// Must be `true`. This is a
    /// [W3C extension](https://w3c.github.io/webcrypto/#iana-section-jwk).
    ext: bool,
}
