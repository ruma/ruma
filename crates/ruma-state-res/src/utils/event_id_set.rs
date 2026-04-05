//! A set of event IDs.

use std::{
    borrow::Borrow,
    collections::{HashSet, hash_set},
    fmt,
    hash::{Hash, RandomState},
    iter::FusedIterator,
};

use ruma_common::EventId;

/// A set of event IDs.
#[derive(Clone, Debug)]
pub struct EventIdSet<E: Borrow<EventId>>(HashSet<E>);

impl<E: Borrow<EventId>> EventIdSet<E> {
    /// Create an empty `EventIdSet`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty `EventIdSet` with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashSet::with_capacity(capacity))
    }

    /// Clears the set, removing all event IDs.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set contains no elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets an iterator over the elements of the set.
    pub fn iter(&self) -> EventIdSetIter<'_, E> {
        EventIdSetIter(self.0.iter())
    }
}

impl<E> EventIdSet<E>
where
    E: Borrow<EventId> + Eq + Hash,
{
    /// Returns `true` if the set contains the specified event ID.
    pub fn contains(&self, event_id: &EventId) -> bool {
        self.0.contains(event_id)
    }

    /// Returns a reference to the event ID in the set, if any, that is equal to the given one.
    pub fn get(&self, event_id: &EventId) -> Option<&E> {
        self.0.get(event_id)
    }

    /// Adds an event ID to the set.
    ///
    /// Returns whether the ID was newly inserted.
    pub fn insert(&mut self, event_id: E) -> bool {
        self.0.insert(event_id)
    }

    /// Removes an event ID from the set.
    ///
    /// Returns whether the ID was present in the set.
    pub fn remove(&mut self, event_id: &EventId) -> bool {
        self.0.remove(event_id)
    }

    /// Removes and returns the event ID in the set, if any, that is equal to the given one.
    pub fn take(&mut self, event_id: &EventId) -> Option<E> {
        self.0.take(event_id)
    }

    /// Visits the values representing the intersection, i.e., the values that are both in self and
    /// other.
    pub fn intersection<'a>(&'a self, other: &'a Self) -> EventIdSetIntersection<'a, E> {
        EventIdSetIntersection(self.0.intersection(&other.0))
    }
}

impl<E: Borrow<EventId>> Default for EventIdSet<E> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<E, const N: usize> From<[E; N]> for EventIdSet<E>
where
    E: Borrow<EventId> + Hash + Eq,
{
    fn from(value: [E; N]) -> Self {
        Self(value.into())
    }
}

impl<E> Extend<E> for EventIdSet<E>
where
    E: Borrow<EventId> + Hash + Eq,
{
    fn extend<T: IntoIterator<Item = E>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl<E> FromIterator<E> for EventIdSet<E>
where
    E: Borrow<EventId> + Hash + Eq,
{
    fn from_iter<T: IntoIterator<Item = E>>(iter: T) -> Self {
        Self(HashSet::from_iter(iter))
    }
}

impl<E: Borrow<EventId>> IntoIterator for EventIdSet<E> {
    type Item = E;
    type IntoIter = EventIdSetIntoIter<E>;

    fn into_iter(self) -> Self::IntoIter {
        EventIdSetIntoIter(self.0.into_iter())
    }
}

impl<'a, E: Borrow<EventId>> IntoIterator for &'a EventIdSet<E> {
    type Item = &'a E;
    type IntoIter = EventIdSetIter<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the values of an [`EventIdSet`].
#[derive(Clone, Debug)]
pub struct EventIdSetIter<'a, E>(hash_set::Iter<'a, E>);

impl<'a, E> Iterator for EventIdSetIter<'a, E> {
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

impl<'a, E> ExactSizeIterator for EventIdSetIter<'a, E> {}

impl<'a, E> FusedIterator for EventIdSetIter<'a, E> {}

/// An iterator over the values of an [`EventIdSet`].
#[derive(Debug)]
pub struct EventIdSetIntoIter<E>(hash_set::IntoIter<E>);

impl<E> Iterator for EventIdSetIntoIter<E> {
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

impl<E> ExactSizeIterator for EventIdSetIntoIter<E> {}

impl<E> FusedIterator for EventIdSetIntoIter<E> {}

/// A lazy iterator producing elements in the intersection of [`EventIdSet`]s.
#[derive(Clone)]
pub struct EventIdSetIntersection<'a, E>(hash_set::Intersection<'a, E, RandomState>);

impl<'a, E> fmt::Debug for EventIdSetIntersection<'a, E>
where
    E: fmt::Debug + Eq + Hash,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EventIdSetIntersection").field(&self.0).finish()
    }
}

impl<'a, E> Iterator for EventIdSetIntersection<'a, E>
where
    E: Eq + Hash,
{
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.0.fold(init, f)
    }
}

impl<'a, E> FusedIterator for EventIdSetIntersection<'a, E> where E: Eq + Hash {}
