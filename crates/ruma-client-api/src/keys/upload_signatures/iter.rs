use std::collections::btree_map;

use serde_json::value::RawValue as RawJsonValue;

use super::v3::SignedKeys;

impl<'a> IntoIterator for &'a SignedKeys {
    type Item = (&'a str, &'a RawJsonValue);
    type IntoIter = SignedKeysIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over signed key IDs and their associated data.
#[derive(Debug)]
pub struct SignedKeysIter<'a>(pub(super) btree_map::Iter<'a, Box<str>, Box<RawJsonValue>>);

impl<'a> Iterator for SignedKeysIter<'a> {
    type Item = (&'a str, &'a RawJsonValue);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(id, val)| (&**id, &**val))
    }
}
