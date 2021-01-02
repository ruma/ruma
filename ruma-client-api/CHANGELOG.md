# 0.10.0 [unreleased]

Bug fixes:

* Fix deserialization of `r0::room::get_room_event::Response`
* More missing fields in `r0::sync::sync_events::Response` can be deserialized
* Fix `get_tags::Response` serialization

Breaking changes:

* Update `contains_url: Option<bool>` in `r0::filter::RoomEventFilter` to
  `url_filter: Option<UrlFilter>`.
* Borrow strings in outgoing requests and responses.
  * Explicit types may have to be updated from `endpoint::Request` to `endpoint::Request<'_>` on
    clients and `endpoint::IncomingRequest` on servers, the other way around for responses.
  * When sending a request or response, you shouldn't have to clone things as much as before. Tip:
    Use clippy to detect now-unnecessary `.into()` conversions.
* Make most types non-exhaustive
  * This means you no longer can construct many of them using struct literals.
  * Instead, constructors are provided.
  * Tip: To set optional fields that aren't set in the constructor, you may find the `assign` crate
    useful.
* Make `avatar_url` in `r0::profile::set_avatar_url::Request` an `Option`
* Update type of `canonical_alias` in `r0::directory::PublicRoomsChunk` from
  `Option<String>` to `Option<RoomAliasId>`
* Update `r0::room::create_room::CreationContent`
  * Change `federated`s type from `Option<bool>` to `bool`
  * Add `predecessor` field
* Update `r0::push::get_pushrules_all` and `r0::push::get_pushrules_global_scope` to use the
  `Ruleset` type from `ruma_common::push` (also available as `ruma::push`)
* Fix event types in `r0::context::get_context`
* Fix event types in `r0::sync::sync_events`
* Update type of `user_id` in `r0::account::whoami` from `String` to `ruma_identifiers::UserId`
* Update type of `limited` in `r0::sync::sync_events::Timeline` from `Option<bool>` to `bool`
* Use `DeviceId` for `device_id` field of `r0::session::login::Response`
* Use `ruma_identifiers::ServerName` instead of `String` for `server_name` fields in the following
  endpoints:
  ```rust
  r0::{
      account::request_openid_token,
      media::{get_content, get_content_as_filename, get_content_thumbnail},
      membership::join_room_by_id_or_alias,
      session::login,
  }
  ```
* Rename `r0::search::search_events::{RoomEventJsons => ResultRoomEvents}`. The previous name was an
  error introduced in a mass search and replace
* `r0::sync::sync_events::SetPresence` has been moved and renamed. Use `presence::PresenceState`
  from `ruma` or `ruma-common`.
* `r0::push::Action` has been moved. Import it from `ruma` or `ruma-common`.
* Update type of `limit` in `r0::user_directory::search_users` from
  `Option<UInt>` to `UInt`
* Rename `r0::message::{create_message_event => send_message_event}`
* Rename `r0::state::{create_state_event_* => send_state_event_*}`
* Replace `r0::keys::{AlgorithmAndDeviceId, KeyAlgorithm}` with
  `ruma_identifiers::{DeviceKeyId, DeviceKeyAlgorithm}`, respectively
* Use `ruma_identifiers::{ServerName, ServerKeyId}` in `signatures` fields of
  `r0::room::membership::ThirdPartySigned`.
* Move `r0::directory::{Filter, PublicRoomsChunk, RoomNetwork}` to
  the `ruma-common` crate
* Replace `r0::room::create_room::InitialStateEvent` with `ruma_events::InitialStateEvent`
* `error::ErrorKind` no longer implements `Copy`, `FromStr`
* Switch from `AnyEvent` to `AnyRoomEvent` in `r0::search::search_events`
* Move `r0::account::request_openid_token::TokenType` to `ruma-common` crate

Improvements:

* Add method `into_event_content` for `r0::room::create_room::CreationContent`
* Add room visibility endpoints: `r0::directory::{get_room_visibility, set_room_visibility}`.
* Add is_empty helpers for structs in `r0::sync::sync_events`
* Add a constructor for request structs of the followign endpoints
  * `r0::room::create_room`
  * `r0::message::get_message_events`
* Add `logout_devices` field to `r0::account::change_password`
* Add `r0::room::aliases` (introduced in r0.6.1)

# 0.9.0

Bug fixes:

* Fix (de)serialization for `r0::media::get_content_thumnail::Response`
* Make `r0::device::get_devices::Response::devices` public

Breaking changes:

* The `event_id` in the response for the message and state sending endpoints is now required
  * r0.6.0 doesn't say they are required, but this has been fixed for the next version of the spec
* Updated the type of `r0::sync::sync_events::DeviceLists` fields
* Change `r0::device::Device` fields according to the spec

Improvements:

* `r0::keys::AlgorithmAndDeviceId` now implements `Display`

# 0.8.0

Breaking changes:

* Update all endpoints to r0.6.0
  * Some of the changes from that might not be listed below, but it should be
    easy to figure out what changed from the documentation and compiler errors
    if you are using any of the affected endpoints.
* Add `server_name` parameter to `r0::join::join_room_by_id_or_alias`
* Modify `r0::account::AuthenticationData`:
  * Rename to `AuthData`
  * Change to an enum to facilitate fallback auth acknowledgements
  * Add `auth_parameters` field
  * Move to `r0::uiaa` module
* Add `room_network` parameter to `r0::directory::get_public_rooms_filtered` to
  represent `include_all_networks` and `third_party_instance_id` Matrix fields
* Update `r0::account::register` endpoint:
  * Remove `bind_email` request field (removed in r0.6.0)
  * Remove `inhibit_login` request field, make `access_token` and `device_id` response fields optional (added in r0.4.0)
  * Remove deprecated `home_server` response field (removed in r0.4.0)
* Update `r0::contact::get_contacts` endpoint to r0.6.0
* Change `UInt` timestamps to `SystemTime` in:
  * `media::get_media_preview::Request`
  * `push::get_notifications::Notification`
  * `server::get_user_info::ConnectionInfo`
  * `device::Device`
* Change all usages of `HashMap` to `BTreeMap`
* Change the messages type that gets sent out using the `r0::client_exchange::send_event_to_device`
  request.
* Add `M_USER_DEACTIVATED` to `error::ErrorKind`
* Make `display_name` field of `r0::membership::joined_events::RoomMember` optional
* Update `r0::search::search_events` to r0.6.0
* Add `account_data` field to `r0::sync::sync_events`
* Rename `r0::client_exchange` to `r0::to_device`

Improvements:

* Add types for User-Interactive Authentication API: `r0::uiaa::{AuthFlow, UiaaInfo, UiaaResponse}`
* Add missing serde attributes to `get_content_thumbnail` query parameters
* Add missing `state` response field to `r0::message::get_message_events`
* Normalize `serde_json` imports
* Remove dependeny on the `url` crate

# 0.7.2

Bug fixes:

* Fix `create_room` requests without an `initial_state` field failing deserialization
* Fix `sync_events` responses without a `device_one_time_keys_count` field failing deserialization

# 0.7.1

Bug fixes:

* Fix deserialization of `sync_events::Request`
* Fix (de)serialization of `sync_events::RoomSummary`

# 0.7.0

Breaking changes:

* Update ruma-api to 0.15.0
* Update ruma-events to 0.18.0
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
