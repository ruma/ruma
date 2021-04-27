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

## Code Map

This section talks briefly about important files and data structures.

### `error`

An enum representing all possible error cases in state-res. Most of the variants are
passing information of failures from other libraries except `Error::NotFound`.
The `NotFound` variant is used whan an event was not in the `event_map`.

### `event_auth`

This module contains all the logic needed to authenticate and verify events.
The main function for authentication is `auth_check`. There are a few checks
that happen to every event and specific checks for some state events.
Each event is authenticated against the state before the event.
The state is built iterativly with each event being applied to the state and
the next checked before being added. 

**Note:** Any type of event can be check, not just state events.

### `room_version`

`RoomVersion` holds information about each room version and is generated from
`RoomVersionId`. During authentication, an event may be verified differently based
on the room version. The `RoomVersion` keeps track of these differences.

### `state_event`

A trait `Event` that allows the state-res library to abstract over the type of an event.
This avoids a lot of unnecessary conversions and gives more flexibility to users.

### `lib`

All the associated functions of `StateResolution` that are needed to resolve state live
here. Everything that is used by `resolve` is exported giving users access to the pieces
of the algorithm.

**Note:** only state events (events that have a state_key field) are allowed to
participate in resolution.


## Testing

state-res has three main test types, event sorting, event authentication, and state
resolution. State resolution tests the whole system. Start by setting up a room with
events and check the resolved state after adding conflicting events.
Event authentication checks that an event passes or fails based on some initial state.
Event sorting tests that given a DAG of events, the events can be predictably sorted.
