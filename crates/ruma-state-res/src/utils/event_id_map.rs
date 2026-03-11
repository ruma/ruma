//! A map of event IDs to a type `V`.

use std::{
    borrow::Borrow,
    collections::{HashMap, hash_map},
    hash::Hash,
    iter::FusedIterator,
    ops::Index,
};

use ruma_common::EventId;

/// A map of event IDs to a type `V`.
#[derive(Clone, Debug)]
pub struct EventIdMap<E: Borrow<EventId>, V>(HashMap<E, V>);

impl<E: Borrow<EventId>, V> EventIdMap<E, V> {
    /// Create an empty `EventIdMap`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty `EventIdMap` with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    /// Clears the map, removing all key-value pairs.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets an iterator over the entries of the map.
    pub fn iter(&self) -> EventIdMapIter<'_, E, V> {
        EventIdMapIter(self.0.iter())
    }

    /// Gets an iterator over the keys of the map.
    pub fn keys(&self) -> EventIdMapKeys<'_, E, V> {
        EventIdMapKeys(self.0.keys())
    }

    /// Gets an iterator over the keys of the map.
    pub fn into_keys(self) -> EventIdMapIntoKeys<E, V> {
        EventIdMapIntoKeys(self.0.into_keys())
    }

    /// Gets an iterator over the values of the map.
    pub fn values(&self) -> EventIdMapValues<'_, E, V> {
        EventIdMapValues(self.0.values())
    }

    /// Gets an iterator over the values of the map.
    pub fn into_values(self) -> EventIdMapIntoValues<E, V> {
        EventIdMapIntoValues(self.0.into_values())
    }
}

impl<E, V> EventIdMap<E, V>
where
    E: Borrow<EventId> + Eq + Hash,
{
    /// Returns `true` if the map contains a value for the specified event ID.
    pub fn contains_event_id(&self, event_id: &EventId) -> bool {
        self.0.contains_key(event_id)
    }

    /// Returns a reference to the value corresponding to given event ID.
    pub fn get(&self, event_id: &EventId) -> Option<&V> {
        self.0.get(event_id)
    }

    /// Returns a mutable reference to the value corresponding to given event ID.
    pub fn get_mut(&mut self, event_id: &EventId) -> Option<&mut V> {
        self.0.get_mut(event_id)
    }

    /// Returns the key-value pair corresponding to the supplied event ID.
    pub fn get_key_value(&self, event_id: &EventId) -> Option<(&E, &V)> {
        self.0.get_key_value(event_id)
    }

    /// Gets the given event ID's corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, event_id: E) -> EventIdMapEntry<'_, E, V> {
        EventIdMapEntry(self.0.entry(event_id))
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this event ID present, `None` is returned.
    ///
    /// If the map did have this event ID present, the value is updated, and the old value is
    /// returned.
    pub fn insert(&mut self, event_id: E, value: V) -> Option<V> {
        self.0.insert(event_id, value)
    }

    /// Removes an event ID from the map, returning the value at the event ID if the event ID was
    /// previously in the map.
    pub fn remove(&mut self, event_id: &EventId) -> Option<V> {
        self.0.remove(event_id)
    }

    /// Removes an event ID from the map, returning the stored event ID and value if the event ID
    /// was previously in the map.
    pub fn remove_entry(&mut self, event_id: &EventId) -> Option<(E, V)> {
        self.0.remove_entry(event_id)
    }
}

impl<E: Borrow<EventId>, V> Default for EventIdMap<E, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<E, V> Index<&EventId> for EventIdMap<E, V>
where
    E: Borrow<EventId> + Hash + Eq,
{
    type Output = V;

    fn index(&self, event_id: &EventId) -> &Self::Output {
        &self.0[event_id]
    }
}

impl<E, V, const N: usize> From<[(E, V); N]> for EventIdMap<E, V>
where
    E: Borrow<EventId> + Hash + Eq,
{
    fn from(value: [(E, V); N]) -> Self {
        Self(value.into())
    }
}

impl<E, V> Extend<(E, V)> for EventIdMap<E, V>
where
    E: Borrow<EventId> + Hash + Eq,
{
    fn extend<T: IntoIterator<Item = (E, V)>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl<E, V> FromIterator<(E, V)> for EventIdMap<E, V>
where
    E: Borrow<EventId> + Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = (E, V)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

impl<E: Borrow<EventId>, V> IntoIterator for EventIdMap<E, V> {
    type Item = (E, V);
    type IntoIter = EventIdMapIntoIter<E, V>;

    fn into_iter(self) -> Self::IntoIter {
        EventIdMapIntoIter(self.0.into_iter())
    }
}

impl<'a, E: Borrow<EventId>, V> IntoIterator for &'a EventIdMap<E, V> {
    type Item = (&'a E, &'a V);
    type IntoIter = EventIdMapIter<'a, E, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the entries of an [`EventIdMap`].
#[derive(Clone, Debug)]
pub struct EventIdMapIter<'a, E, V>(hash_map::Iter<'a, E, V>);

impl<'a, E, V> Iterator for EventIdMapIter<'a, E, V> {
    type Item = (&'a E, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize {
        self.0.len()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, f)
    }
}

impl<'a, E, V> ExactSizeIterator for EventIdMapIter<'a, E, V> {}

impl<'a, E, V> FusedIterator for EventIdMapIter<'a, E, V> {}

/// An iterator over the entries of an [`EventIdMap`].
#[derive(Debug)]
pub struct EventIdMapIntoIter<E, V>(hash_map::IntoIter<E, V>);

impl<E, V> Iterator for EventIdMapIntoIter<E, V> {
    type Item = (E, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize {
        self.0.len()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, f)
    }
}

impl<E, V> ExactSizeIterator for EventIdMapIntoIter<E, V> {}

impl<E, V> FusedIterator for EventIdMapIntoIter<E, V> {}

/// An iterator over the keys of an [`EventIdMap`].
#[derive(Clone, Debug)]
pub struct EventIdMapKeys<'a, E, V>(hash_map::Keys<'a, E, V>);

impl<'a, E, V> Iterator for EventIdMapKeys<'a, E, V> {
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize {
        self.0.len()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, f)
    }
}

impl<'a, E, V> ExactSizeIterator for EventIdMapKeys<'a, E, V> {}

impl<'a, E, V> FusedIterator for EventIdMapKeys<'a, E, V> {}

/// An iterator over the keys of an [`EventIdMap`].
#[derive(Debug)]
pub struct EventIdMapIntoKeys<E, V>(hash_map::IntoKeys<E, V>);

impl<E, V> Iterator for EventIdMapIntoKeys<E, V> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize {
        self.0.len()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, f)
    }
}

impl<E, V> ExactSizeIterator for EventIdMapIntoKeys<E, V> {}

impl<E, V> FusedIterator for EventIdMapIntoKeys<E, V> {}

/// An iterator over the values of an [`EventIdMap`].
#[derive(Clone, Debug)]
pub struct EventIdMapValues<'a, E, V>(hash_map::Values<'a, E, V>);

impl<'a, E, V> Iterator for EventIdMapValues<'a, E, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize {
        self.0.len()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, f)
    }
}

impl<'a, E, V> ExactSizeIterator for EventIdMapValues<'a, E, V> {}

impl<'a, E, V> FusedIterator for EventIdMapValues<'a, E, V> {}

/// An iterator over the values of an [`EventIdMap`].
#[derive(Debug)]
pub struct EventIdMapIntoValues<E, V>(hash_map::IntoValues<E, V>);

impl<E, V> Iterator for EventIdMapIntoValues<E, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize {
        self.0.len()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, f)
    }
}

impl<E, V> ExactSizeIterator for EventIdMapIntoValues<E, V> {}

impl<E, V> FusedIterator for EventIdMapIntoValues<E, V> {}

/// A view into a single entry in an [`EventIdMap`].
#[derive(Debug)]
pub struct EventIdMapEntry<'a, E: Borrow<EventId>, V>(hash_map::Entry<'a, E, V>);

impl<'a, E: Borrow<EventId>, V> EventIdMapEntry<'a, E, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns a mutable
    /// reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.0.or_insert(default)
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        self.0.or_insert_with(default)
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    ///
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function a reference to the key that was moved during the .entry(key) method call.
    pub fn or_insert_with_key<F: FnOnce(&E) -> V>(self, default: F) -> &'a mut V {
        self.0.or_insert_with_key(default)
    }

    /// Sets the value of the entry, and returns a mutable reference to the value.
    pub fn insert_entry(self, value: V) -> &'a mut V {
        self.0.insert_entry(value).into_mut()
    }
}

impl<'a, E: Borrow<EventId>, V: Default> EventIdMapEntry<'a, E, V> {
    /// Ensures a value is in the entry by inserting the default value if empty, and returns a
    /// mutable reference to the value in the entry.
    pub fn or_default(self) -> &'a mut V {
        self.0.or_default()
    }
}
