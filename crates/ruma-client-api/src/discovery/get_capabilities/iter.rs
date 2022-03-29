//! Iterator implementation for `Capabilities`

use std::{borrow::Cow, collections::btree_map};

use serde_json::Value as JsonValue;

use super::Capabilities;

/// Reference to a capability.
#[derive(Debug)]
pub struct CapabilityRef<'a> {
    name: &'a str,
    value: Option<&'a JsonValue>,
    caps: &'a Capabilities,
}

impl<'a> CapabilityRef<'a> {
    /// Get name of the capability.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Get value of the capability.
    pub fn value(&self) -> Cow<'a, JsonValue> {
        match self.value {
            // unknown capability from btreemap iterator
            Some(val) => Cow::Borrowed(val),
            // O(1) lookup of known capability
            None => self.caps.get(self.name).unwrap(),
        }
    }
}

/// An iterator over capabilities.
#[derive(Debug)]
pub struct CapabilitiesIter<'a> {
    /// Reference to Capabilities
    caps: &'a Capabilities,
    /// Current position of the iterator
    pos: usize,
    /// Iterator for custom capabilities
    custom_caps_iterator: btree_map::Iter<'a, String, JsonValue>,
}

impl<'a> CapabilitiesIter<'a> {
    /// Creates a new CapabilitiesIter
    pub(super) fn new(caps: &'a Capabilities) -> Self {
        Self { caps, pos: 0, custom_caps_iterator: caps.custom_capabilities.iter() }
    }
}

impl<'a> Iterator for CapabilitiesIter<'a> {
    type Item = CapabilityRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pos {
            0 => {
                self.pos += 1;
                Some(CapabilityRef { name: "m.change_password", value: None, caps: self.caps })
            }
            1 => {
                self.pos += 1;
                Some(CapabilityRef { name: "m.room_versions", value: None, caps: self.caps })
            }
            2 => {
                self.pos += 1;
                Some(CapabilityRef { name: "m.set_displayname", value: None, caps: self.caps })
            }
            3 => {
                self.pos += 1;
                Some(CapabilityRef { name: "m.set_avatar_url", value: None, caps: self.caps })
            }
            4 => {
                self.pos += 1;
                Some(CapabilityRef { name: "m.3pid_changes", value: None, caps: self.caps })
            }
            _ => self.custom_caps_iterator.next().map(|(name, value)| CapabilityRef {
                name,
                value: Some(value),
                caps: self.caps,
            }),
        }
    }
}
