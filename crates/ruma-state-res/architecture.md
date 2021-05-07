# Architecture

This document describes the high-level architecture of state-res.
If you want to familiarize yourself with the code base, you are just in the right place!

## Overview

The state-res crate provides all the necessary algorithms to resolve the state of a
room according to the Matrix spec. Given sets of state and the complete authorization
chain, a final resolved state is calculated.

The state sets (`BTreeMap<(EventType, StateKey), EventId>`) can be the state of a room
according to different servers or at different points in time. The authorization chain
is the recursive set of all events that authorize events that come after.
Any event that can be referenced needs to be available in the `event_map` argument,
or the call fails. The `StateResolution` struct keeps no state and is only a
collection of associated functions.

## Important Terms

 - **event** In state-res this refers to a **P**ersistent **D**ata **U**nit which
 represents the event and keeps metadata used for resolution
 - **state resolution** The process of calculating the final state of a DAG from
 conflicting input DAGs

## Code Map

This section talks briefly about important files and data structures.

### `error`

An enum representing all possible error cases in state-res. Most of the variants are
passing information of failures from other libraries except `Error::NotFound`.
The `NotFound` variant is used when an event was not in the `event_map`.

### `event_auth`

This module contains all the logic needed to authenticate and verify events.
The main function for authentication is `auth_check`. There are a few checks
that happen to every event and specific checks for some state events.
Each event is authenticated against the state before the event.
The state is built iteratively with each successive event being checked against
the current state then added. 

**Note:** Any type of event can be check, not just state events.

### `state_event`

A trait called `Event` that allows the state-res library to take any PDU type the user
supplies. The main `StateResolution::resolve` function can resolve any user-defined
type that satisfies `Event`. This avoids a lot of unnecessary conversions and
gives more flexibility to users.

### `lib`

All the associated functions of `StateResolution` that are needed to resolve state live
here. The focus is `StateResolution::resolve`, given a DAG and new events
`resolve` calculates the end state of the DAG. Everything that is used by `resolve`
is exported giving users access to the pieces of the algorithm.

**Note:** only state events (events that have a state_key field) are allowed to
participate in resolution.

## Testing

state-res has three main test types: event sorting, event authentication, and state
resolution. State resolution tests the whole system. Start by setting up a room with
events and check the resolved state after adding conflicting events.
Event authentication checks that an event passes or fails based on some initial state.
Event sorting tests that given a DAG of events, the events can be predictably sorted.
