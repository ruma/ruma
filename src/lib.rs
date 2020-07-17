use std::{collections::BTreeMap, time::SystemTime};

use petgraph::Graph;
use ruma::{
    events::{
        room::{self},
        AnyStateEvent, AnyStrippedStateEvent, AnySyncStateEvent, EventType,
    },
    identifiers::{EventId, RoomId, RoomVersionId},
};
use serde::{Deserialize, Serialize};

mod state_event;
mod state_store;

pub use state_event::StateEvent;
pub use state_store::StateStore;

pub enum ResolutionResult {
    Conflicted(Vec<StateMap<EventId>>),
    Resolved(Vec<StateMap<EventId>>),
}

/// A mapping of event type and state_key to some value `T`, usually an `EventId`.
pub type StateMap<T> = BTreeMap<(EventType, String), T>;

/// A mapping of `EventId` to `T`, usually a `StateEvent`.
pub type EventMap<T> = BTreeMap<EventId, T>;

#[derive(Debug, Default, Deserialize, Serialize)] // TODO make the ser/de impls useful
pub struct StateResolution {
    // TODO remove pub after initial testing
    /// The set of resolved events over time.
    pub resolved_events: Vec<StateEvent>,
    /// The resolved state, kept to have easy access to the last resolved
    /// layer of state.
    pub state: BTreeMap<EventType, BTreeMap<String, StateEvent>>,
    /// The graph of authenticated events, kept to find the most recent auth event
    /// in a chain for incoming state sets.
    pub auth_graph: BTreeMap<EventId, Vec<StateMap<EventId>>>,
    /// The last known point in the state graph.
    pub most_recent_resolved: Option<(EventType, String)>,

    // fields for temp storage during resolution
    pub conflicting_events: Vec<StateEvent>,
}

impl StateResolution {
    /// Resolve sets of state events as they come in. Internally `StateResolution` builds a graph
    /// and an auth chain to allow for state conflict resolution.
    pub fn resolve(
        &mut self,
        room_id: &RoomId,
        room_version: &RoomVersionId,
        state_sets: Vec<StateMap<EventId>>,
        store: &mut dyn StateStore,
        // TODO actual error handling (`thiserror`??)
    ) -> Result<ResolutionResult, serde_json::Error> {
        let mut event_map = EventMap::new();
        // split non-conflicting and conflicting state
        let (clean, mut conflicting) = self.seperate(&state_sets);

        if conflicting.is_empty() {
            return Ok(ResolutionResult::Resolved(clean));
        }

        // the set of auth events that are not common across server forks
        let mut auth_diff = self.get_auth_chain_diff(&state_sets, &mut event_map, store)?;

        // add the auth_diff to conflicting now we have a full set of conflicting events
        auth_diff.extend(conflicting.iter().flat_map(|map| map.values().cloned()));
        let all_conflicted = auth_diff;

        let all_conflicted = conflicting;

        let power_events = all_conflicted
            .iter()
            .filter(is_power_event)
            .flat_map(|map| map.values())
            .cloned()
            .collect::<Vec<_>>();

        // sort the power events based on power_level/clock/event_id and outgoing/incoming edges
        let sorted_power_levels = self.revers_topological_power_sort(
            room_id,
            &power_events,
            &mut event_map,
            store,
            &all_conflicted,
        );

        // sequentially auth check each event.
        let resolved = self.iterative_auth_check(
            room_id,
            room_version,
            &power_events,
            &clean,
            &mut event_map,
            store,
        );

        // TODO return something not a place holder
        Ok(ResolutionResult::Resolved(vec![]))
    }

    fn seperate(
        &mut self,
        state_sets: &[StateMap<EventId>],
    ) -> (Vec<StateMap<EventId>>, Vec<StateMap<EventId>>) {
        panic!()
    }

    /// Returns a Vec of deduped EventIds that appear in some chains but no others.
    fn get_auth_chain_diff(
        &mut self,
        state_sets: &[StateMap<EventId>],
        event_map: &EventMap<StateEvent>,
        store: &mut dyn StateStore,
    ) -> Result<Vec<EventId>, serde_json::Error> {
        panic!()
    }

    fn revers_topological_power_sort(
        &mut self,
        room_id: &RoomId,
        power_events: &[EventId],
        event_map: &EventMap<StateEvent>,
        store: &mut dyn StateStore,
        conflicted_set: &[StateMap<EventId>],
    ) -> Vec<StateEvent> {
        panic!()
    }

    fn iterative_auth_check(
        &mut self,
        room_id: &RoomId,
        room_version: &RoomVersionId,
        power_events: &[EventId],
        unconflicted_state: &[StateMap<EventId>],
        event_map: &EventMap<StateEvent>,
        store: &mut dyn StateStore,
    ) -> Vec<StateEvent> {
        panic!()
    }
}

pub fn is_power_event(event: &&StateMap<EventId>) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
}
