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

// We want to yield to the reactor occasionally during state res when dealing
// with large data sets, so that we don't exhaust the reactor. This is done by
// yielding to reactor during loops every N iterations.
const _YIELD_AFTER_ITERATIONS: usize = 100;

pub enum ResolutionResult {
    Conflicted(Vec<StateMap<EventId>>),
    Resolved(StateMap<EventId>),
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
        state_sets: &[StateMap<EventId>],
        store: &dyn StateStore,
        // TODO actual error handling (`thiserror`??)
    ) -> Result<ResolutionResult, String> {
        tracing::debug!("State resolution starting");

        let mut event_map = EventMap::new();
        // split non-conflicting and conflicting state
        let (clean, conflicting) = self.seperate(&state_sets);

        if conflicting.is_empty() {
            return Ok(ResolutionResult::Resolved(clean));
        }

        tracing::debug!("computing {} conflicting events", conflicting.len());

        // the set of auth events that are not common across server forks
        let mut auth_diff = self.get_auth_chain_diff(&state_sets, &mut event_map, store)?;

        // add the auth_diff to conflicting now we have a full set of conflicting events
        auth_diff.extend(conflicting.values().cloned().flatten());
        let all_conflicted = auth_diff;

        tracing::debug!("full conflicted set is {} events", all_conflicted.len());

        let events = store
            .get_events(
                &all_conflicted
                    .iter()
                    // we only want the events we don't know about yet
                    .filter(|id| !event_map.contains_key(id))
                    .cloned()
                    .collect::<Vec<_>>(),
            )
            .unwrap();

        for event in event_map.values() {
            if event.room_id() != Some(room_id) {
                return Err(format!(
                    "resolving event {} in room {}, when correct room is {}",
                    event
                        .event_id()
                        .map(|id| id.as_str())
                        .unwrap_or("`unknown`"),
                    event.room_id().map(|id| id.as_str()).unwrap_or("`unknown`"),
                    room_id.as_str()
                ));
            }
        }
        // TODO throw error if event is not for this room
        // TODO make sure each conflicting event is in?? event_map `{eid for eid in full_conflicted_set if eid in event_map}`

        let power_events = all_conflicted
            .iter()
            .filter(|id| is_power_event(id, store))
            .cloned()
            .collect::<Vec<_>>();

        // sort the power events based on power_level/clock/event_id and outgoing/incoming edges
        let mut sorted_power_levels = self.revers_topological_power_sort(
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
            &sorted_power_levels,
            &clean,
            &mut event_map,
            store,
        );

        // At this point the power_events have been resolved we now have to
        // sort the remaining events using the mainline of the resolved power level.
        sorted_power_levels.dedup();
        let deduped_power_ev = sorted_power_levels;

        let events_to_resolve = all_conflicted
            .iter()
            .filter(|id| deduped_power_ev.contains(id))
            .cloned()
            .collect::<Vec<_>>();

        let power_event = resolved.get(&(EventType::RoomPowerLevels, "".into()));

        let sorted_left_events =
            self.mainline_sort(room_id, &events_to_resolve, power_event, &event_map, store);

        let mut resolved_state = self.iterative_auth_check(
            room_id,
            room_version,
            &sorted_left_events,
            &resolved,
            &mut event_map,
            store,
        );

        // add unconflicted state to the resolved state
        resolved_state.extend(clean);

        // TODO return something not a place holder
        Ok(ResolutionResult::Resolved(resolved_state))
    }

    /// Split the events that have no conflicts from those that are conflicting.
    ///
    /// The tuple looks like `(unconflicted, conflicted)`.
    fn seperate(
        &mut self,
        state_sets: &[StateMap<EventId>],
    ) -> (StateMap<EventId>, StateMap<Vec<EventId>>) {
        let mut unconflicted_state = StateMap::new();
        let mut conflicted_state = StateMap::new();

        for key in state_sets.iter().flat_map(|map| map.keys()) {
            let mut event_ids = state_sets
                .iter()
                .flat_map(|map| map.get(key).cloned())
                .collect::<Vec<EventId>>();

            if event_ids.len() == 1 {
                // unwrap is ok since we know the len is 1
                unconflicted_state.insert(key.clone(), event_ids.pop().unwrap());
            } else {
                conflicted_state.insert(key.clone(), event_ids);
            }
        }

        (unconflicted_state, conflicted_state)
    }

    /// Returns a Vec of deduped EventIds that appear in some chains but no others.
    fn get_auth_chain_diff(
        &mut self,
        state_sets: &[StateMap<EventId>],
        event_map: &EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> Result<Vec<EventId>, String> {
        tracing::debug!("calculating auth chain difference");
        panic!()
    }

    fn revers_topological_power_sort(
        &mut self,
        room_id: &RoomId,
        power_events: &[EventId],
        event_map: &EventMap<StateEvent>,
        store: &dyn StateStore,
        conflicted_set: &[EventId],
    ) -> Vec<EventId> {
        tracing::debug!("reverse topological sort of power events");
        panic!()
    }

    fn iterative_auth_check(
        &mut self,
        room_id: &RoomId,
        room_version: &RoomVersionId,
        power_events: &[EventId],
        unconflicted_state: &StateMap<EventId>,
        event_map: &EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> StateMap<EventId> {
        tracing::debug!("starting iter auth check");
        panic!()
    }

    /// Returns the sorted `to_sort` list of `EventId`s based on a mainline sort using
    /// the `resolved_power_level`.
    fn mainline_sort(
        &mut self,
        room_id: &RoomId,
        to_sort: &[EventId],
        resolved_power_level: Option<&EventId>,
        event_map: &EventMap<StateEvent>,
        store: &dyn StateStore,
    ) -> Vec<EventId> {
        tracing::debug!("mainline sort of remaining events");
        // There can be no EventId's to sort, bail.
        if to_sort.is_empty() {
            return vec![];
        }

        let mut mainline = vec![];
        let mut pl = resolved_power_level.cloned();
        let mut idx = 0;
        while let Some(p) = pl {
            mainline.push(p.clone());
            // We don't need the actual pl_ev here since we delegate to the store
            let auth_events = store.auth_event_ids(room_id, &p).unwrap();
            pl = None;
            for aid in auth_events {
                let ev = store.get_event(&aid).unwrap();
                if ev.is_type_and_key(EventType::RoomPowerLevels, "") {
                    pl = Some(aid);
                    break;
                }
            }
            // We yield occasionally when we're working with large data sets to
            // ensure that we don't block the reactor loop for too long.
            if idx != 0 && idx % _YIELD_AFTER_ITERATIONS == 0 {
                // yield clock.sleep(0)
            }
            idx += 1;
        }

        let mainline_map = mainline
            .iter()
            .enumerate()
            .map(|(idx, eid)| ((*eid).clone(), idx))
            .collect::<BTreeMap<_, _>>();
        let mut sort_event_ids = to_sort.to_vec();

        let mut order_map = BTreeMap::new();
        for (idx, ev_id) in to_sort.iter().enumerate() {
            let depth = self.get_mainline_depth(
                room_id,
                event_map.get(ev_id).cloned(),
                &mainline_map,
                store,
            );
            order_map.insert(
                ev_id,
                (
                    depth,
                    event_map.get(ev_id).map(|ev| ev.origin_server_ts()),
                    ev_id, // TODO should this be a &str to sort lexically??
                ),
            );

            // We yield occasionally when we're working with large data sets to
            // ensure that we don't block the reactor loop for too long.
            if idx % _YIELD_AFTER_ITERATIONS == 0 {
                // yield clock.sleep(0)
            }
        }

        // sort the event_ids by their depth, timestamp and EventId
        sort_event_ids.sort_by_key(|sort_id| order_map.get(sort_id).unwrap());

        sort_event_ids
    }

    fn get_mainline_depth(
        &mut self,
        room_id: &RoomId,
        mut event: Option<StateEvent>,
        mainline_map: &EventMap<usize>,
        store: &dyn StateStore,
    ) -> usize {
        while let Some(sort_ev) = event {
            if let Some(id) = sort_ev.event_id() {
                if let Some(depth) = mainline_map.get(id) {
                    return *depth;
                }
            }

            let auth_events = if let Some(id) = sort_ev.event_id() {
                store.auth_event_ids(room_id, id).unwrap()
            } else {
                vec![]
            };
            event = None;

            for aid in auth_events {
                let aev = store.get_event(&aid).unwrap();
                if aev.is_type_and_key(EventType::RoomPowerLevels, "") {
                    event = Some(aev);
                    break;
                }
            }
        }
        // Did not find a power level event so we default to zero
        0
    }
}

pub fn is_power_event(event_id: &EventId, store: &dyn StateStore) -> bool {
    match store.get_event(event_id) {
        Ok(state) => state.is_power_event(),
        _ => false, // TODO this shouldn't eat errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
