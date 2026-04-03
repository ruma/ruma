#![allow(clippy::exhaustive_structs)]

use serde::{Serialize, Serializer, ser::SerializeMap};

use super::{RetainKey, RetainedKeys, retained_event_keys};
use crate::{CanonicalJsonObject, CanonicalJsonValue, room_version_rules::RedactionRules};

/// [`CanonicalJsonObject`] serializer that redacts fields on the fly.
///
/// The main use case for this is to compute the hashes or signatures of an event, where the event
/// needs to be redacted and have a few other fields removed.
///
/// This avoids having to `.clone()` a `CanonicalJsonObject`, and then to
/// [`redact()`](super::redact) it and potentially remove other fields, and serialize it with
/// [`serde_json::to_vec()`].
#[derive(Debug, Clone, Copy, Default)]
pub struct RedactingSerializer<'a> {
    /// The redaction rules to apply, if any.
    rules: Option<&'a RedactionRules>,

    /// Custom fields to redact at the root of the object.
    custom_redacted_root_fields: &'a [&'a str],
}

impl<'a> RedactingSerializer<'a> {
    /// Construct a new `RedactingSerializer` that doesn't redact anything.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add the redaction rules to apply.
    ///
    /// If this is set, the object to serialize must be an event with a `type` field.
    pub fn rules(mut self, rules: &'a RedactionRules) -> Self {
        self.rules = Some(rules);
        self
    }

    /// Add custom fields to redact at the root of the object.
    pub fn custom_redacted_root_fields(mut self, fields: &'a [&'a str]) -> Self {
        self.custom_redacted_root_fields = fields;
        self
    }

    /// Serialize the given object while redacting it.
    pub fn serialize(&self, object: &CanonicalJsonObject) -> Result<String, serde_json::Error> {
        let retained_keys = self
            .rules
            .map(|rules| {
                retained_event_keys(object)
                    .map(|retained_keys| RetainedKeysWithRules { retained_keys, rules })
            })
            .transpose()
            .map_err(serde::ser::Error::custom)?;

        serde_json::to_string(&RedactingObjectSerializer {
            object,
            retained_keys,
            custom_redacted_fields: self.custom_redacted_root_fields,
        })
    }
}

/// Wrapper around [`CanonicalJsonObject`] that redacts fields during serialization.
struct RedactingObjectSerializer<'a> {
    object: &'a CanonicalJsonObject,
    retained_keys: Option<RetainedKeysWithRules<'a>>,
    custom_redacted_fields: &'a [&'a str],
}

impl<'a> Serialize for RedactingObjectSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serialize_map = serializer.serialize_map(None)?;

        for (key, value) in self.object.iter() {
            if self.custom_redacted_fields.contains(&key.as_str()) {
                continue;
            }

            if let Some(RetainedKeysWithRules { retained_keys, rules }) = &self.retained_keys {
                if let RetainKey::Yes { child_retained_keys } =
                    retained_keys.should_retain_key(rules, key)
                {
                    // We only support recursive redacting for objects.
                    if let Some(retained_keys) = child_retained_keys
                        && let CanonicalJsonValue::Object(child_object) = value
                    {
                        serialize_map.serialize_entry(
                            key,
                            &Self {
                                object: child_object,
                                retained_keys: Some(RetainedKeysWithRules { retained_keys, rules }),
                                custom_redacted_fields: &[],
                            },
                        )?;
                    } else {
                        serialize_map.serialize_entry(key, value)?;
                    }
                }
            } else {
                serialize_map.serialize_entry(key, value)?;
            }
        }

        serialize_map.end()
    }
}

/// The retained keys for an object and the redaction rules to apply.
struct RetainedKeysWithRules<'a> {
    /// The keys to retain for the object.
    retained_keys: RetainedKeys,

    /// The redaction rules to apply.
    rules: &'a RedactionRules,
}
