# [unreleased]

Improvements:

* Add an `Error` type that represents the well-known errors in the client-server API
  * the response deserialization code will try to create an instance of this type from http responses that indicate an error
* Add OpenID token request endpoint.
* Add `r0::client_exchange::send_event_to_device` (introduced in r0.3.0)
* Add endpoints to retrieve account_data (introduced in r0.5.0)
* Add media endpoints: `r0::media::{get_media_config, get_media_preview, get_content_as_filename}`
* Add `unstable_features` to `unversioned::get_supported_versions` (introduced in r0.5.0)
* Add request and response parameters for `r0::account::deactivate`
* Add `r0::session::sso_login` (introduced in r0.5.0)
* Add `filter` type for `r0::context::get_context`

Breaking changes:

* Update ruma-api to 0.15.0
* Fix `r0::session::get_login_types`
* Add `allow_remote` parameter to `r0::media::get_content`
* Add missing parameters for `r0::room::create_room`
* Moved `r0::room::create_room::Invite3pid` to `r0::membership::Invite3pid`
* Replaced `user_id` parameter of `r0::membership::invite_user` with `recipient`
  to allow invitation of users by either Matrix or third party identifiers.
* Remove deprecated endpoint `r0::contact::create_contact` (deprecated in r0.6.0)
* Add lazy-loading options to `r0::filter::RoomEventFilter` (introduced in r0.5.0)
* Change type for `limit` request parameter of `r0::context::get_context` from `u8` to `Option<js_int::UInt>`
* Use `std::time::Duration` for appropriate fields on several endpoints:
  ```
  r0::{
      account::request_openid_token,
      keys::{claim_keys, get_keys},
      presence::get_presence,
      sync::sync_events,
      typing::create_typing_event,
      voip::get_turn_server_info
  }
  ```

# 0.6.0

Breaking changes:

* Update ruma-api to 0.13.0
* Our Minimum Supported Rust Version is now 1.40.0
* Remove presence list endpoints `r0::presence::{get_subscribed_presences, update_presence_subscriptions}` (removed in r0.5.0)
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
* Update endpoints for requesting account management tokens via email:
  * Move `r0::account::request_password_change_token` to `r0::account::request_password_change_token_via_email`
  * Move `r0::account::request_register_token` to `r0::account::request_registration_token_via_email`
  * Modify `r0::account::request_registration_token_via_email` not to be rate-limited and require authentication
* Merge duplicate enums `r0::contact::get_contact::Medium` and `r0::session::login::Medium` and move them to `r0::thirdparty`

Improvements:

* Add `r0::device` endpoints
* Add `r0::room::get_room_event` (introduced in r0.4.0)
* Add `r0::read_marker::set_read_marker` (introduced in r0.4.0)
* Add `r0::capabilities::get_capabilities` (introduced in r0.5.0)
* Add `r0::keys` endpoints (introduced in r0.3.0)
* Add `r0::session::get_login_types` (introduced in r0.4.0)
* Add `r0::account::get_username_availability` (introduced in r0.4.0)
* Add endpoints to request management tokens (introduced upstream in r0.4.0):
  * `r0::account::request_3pid_management_token_via_msisdn`
  * `r0::account::request_password_change_token_via_msisdn`
  * `r0::account::request_registration_token_via_msisdn`
  * `r0::acount::request_3pid_management_token_via_email`
* Update `r0::presence_get_presence` from r0.4.0 to r0.6.0
* Add `r0::account::bind_3pid`
* Add `r0::account::delete_3pid`
* Add `r0::account::unbind_3pid`
* Add `r0::push` endpoints
* Add `r0::room::upgrade_room` (introduced upstream in r0.5.0)

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
