# [unreleased]

Breaking changes:

* Remove presence list endpoints `r0::presence::{get_subscribed_presences, update_presence_subscriptions}` (removed in 0.5.0)
* Refactor `r0::send` endpoints and remove module:
  * Move `r0::send::send_message_event` to `r0::message::create_message_event`
  * Move `r0::send::send_state_event_for_empty_key` to `r0::state:create_state_event_for_empty_key`
  * Move `r0::send::send_state_event_for_key` to `r0::state:create_state_event_for_key`
* Refactor `r0::sync` endpoints:
  * Move `r0::sync::get_member_events` to `r0::membership::get_member_events`
  * Move `r0::sync::get_message_events` to `r0::message::get_message_events`
  * Move `r0::sync::get_state_events` to `r0::state::get_state_events`
  * Move `r0::sync::get_state_events_for_empty_key` to `r0::state::get_state_events_for_empty_key`
  * Move `r0::sync::get_state_events_for_key` to `r0::state::get_state_events_for_key`

Improvements:

* Add `r0::device` endpoints
* Add `r0::room::get_room_event` (introduced in r0.4.0)
* Add `r0::read_marker::set_read_marker` (introduced in r0.4.0)
* Add `r0::capabilities::get_capabilities` (introduced in r0.5.0)
* Add `r0::keys` endpoints (introduced in r0.3.0)

# 0.5.0

Breaking changes:

* Our Minimum Supported Rust Version is now 1.39.0
* Update ruma-api from 0.11.0 to 0.12.0
* Move `r0::directory::get_public_rooms::PublicRoomsChunk` to `r0::directory::PublicRoomsChunk`
* Move `r0::room::create_room::Visibility` to `r0::room::Visibility`
* Move `r0::account::register::AuthenticationData` to `r0::account::AuthenticationData`

Improvements:

* Update `r0::directory::get_public_rooms` from r0.3.0 to r0.6.0
* Add `r0::directory::get_public_rooms_filtered` (introduced upstream in r0.3.0)
* Add `filter` optional parameter to `r0::sync::get_message_events` (introduced upstream in r0.3.0)
* Add `r0::appservice::set_room_visibility` (part of application service extensions for the client-server API)
* Add `contains_url` to `r0::filter::RoomEventFilter` (introduced upstream in r0.3.0)
* Update `r0::account::change_password` from r0.3.0 to r0.6.0
  * Add optional `auth` field
