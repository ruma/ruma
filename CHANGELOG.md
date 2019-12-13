# [unreleased]

# r0.5.0

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
