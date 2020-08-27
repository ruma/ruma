use std::collections::BTreeSet;

use ruma::identifiers::{EventId, RoomId};

use crate::{Result, StateEvent};

pub trait StateStore {
    /// Return a single event based on the EventId.
    fn get_event(&self, room_id: &RoomId, event_id: &EventId) -> Result<StateEvent>;

    /// Returns the events that correspond to the `event_ids` sorted in the same order.
    fn get_events(&self, room_id: &RoomId, event_ids: &[EventId]) -> Result<Vec<StateEvent>> {
        let mut events = vec![];
        for id in event_ids {
            events.push(self.get_event(room_id, id)?);
        }
        Ok(events)
    }

    /// Returns a Vec of the related auth events to the given `event`.
    fn auth_event_ids(&self, room_id: &RoomId, event_ids: &[EventId]) -> Result<Vec<EventId>> {
        let mut result = vec![];
        let mut stack = event_ids.to_vec();

        // DFS for auth event chain
        while !stack.is_empty() {
            let ev_id = stack.pop().unwrap();
            if result.contains(&ev_id) {
                continue;
            }

            result.push(ev_id.clone());

            let event = self.get_event(room_id, &ev_id).unwrap();

            stack.extend(event.auth_events());
        }

        Ok(result)
    }

    /// Returns a Vec<EventId> representing the difference in auth chains of the given `events`.
    fn auth_chain_diff(
        &self,
        room_id: &RoomId,
        event_ids: Vec<Vec<EventId>>,
    ) -> Result<Vec<EventId>> {
        let mut chains = vec![];
        for ids in event_ids {
            // TODO state store `auth_event_ids` returns self in the event ids list
            // when an event returns `auth_event_ids` self is not contained
            let chain = self
                .auth_event_ids(room_id, &ids)?
                .into_iter()
                .collect::<BTreeSet<_>>();
            chains.push(chain);
        }

        if let Some(chain) = chains.first() {
            let rest = chains.iter().skip(1).flatten().cloned().collect();
            let common = chain.intersection(&rest).collect::<Vec<_>>();

            Ok(chains
                .iter()
                .flatten()
                .filter(|id| !common.contains(&id))
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect())
        } else {
            Ok(vec![])
        }
    }
}
