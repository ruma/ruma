use std::{borrow::Cow, collections::btree_map};

use serde_json::Value as JsonValue;

use super::get_capabilities::Capabilities;

/// Reference to a capability.
#[derive(Debug)]
pub struct CapabilityRef<'a> {
    name: &'a str,
    value: Option<&'a JsonValue>,
    caps: &'a Capabilities,
}

impl<'a> CapabilityRef<'a> {
    pub fn name(&self) -> &'a str {
        self.name
    }

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
    pub(super) caps: &'a Capabilities,
    pub(super) pos: usize,
    pub(super) custom_caps_iterator: btree_map::Iter<'a, String, JsonValue>,
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
            _ => self.custom_caps_iterator.next().map(|(name, value)| CapabilityRef {
                name,
                value: Some(value),
                caps: self.caps,
            }),
        }
    }
}
