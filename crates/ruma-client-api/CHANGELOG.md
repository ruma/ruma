# [unreleased]

# 0.18.0

Bug fixes:

- Don't require the `failures` field in the
  `ruma_client_api::keys::upload_signatures::Response` type.
- `sync::sync_events::v3::Timeline::is_empty` now returns `false` when the
  `limited` or `prev_batch` fields are set.
- `login_fallback::Response` now returns the proper content type
- `sso_login[_with_provider]` responses now use the proper HTTP status code.

Breaking changes:

- The conversion from `PushRule` to `ConditionalPushRule` is infallible since
  the `conditions` field is optional.
  - `MissingConditionsError` was removed.
- The `ts` field in `Request` for `get_media_preview` is now `Option`.
- The query parameter of `check_registration_token_validity` endpoint
  has been renamed from `registration_token` to `token`
- `Error` is now non-exhaustive.
- `ErrorKind::Forbidden` is now a non-exhaustive struct variant that can be
  constructed with `ErrorKind::forbidden()`.
- The `retry_after_ms` field of `ErrorKind::LimitExceeded` was renamed to
  `retry_after` and is now an `Option<RetryAfter>`, to add support for the
  Retry-After header, according to MSC4041 / Matrix 1.10
- Make `get_uiaa_fallback::v3::Response` an enum for a redirect or an HTML page.
  It will now return the proper status code and headers depending on the variant
  used.
- The http crate had a major version bump to version 1.1

Improvements:

- Point links to the Matrix 1.10 specification
- Add the `get_authentication_issuer` endpoint from MSC2965 behind the
  `unstable-msc2965` feature.
- Add `error_kind` accessor method to `ruma_client_api::Error`
- Add `FromHttpResponseErrorExt` trait that adds an `error_kind` accessor to
  `FromHttpResponseError<ruma_client_api::Error>`
- Add deprecated `user` fields for `m.login.password` and `m.login.appservice`
  login types.
- Add deprecated `address` and `medium` 3PID fields for `m.login.password`
  login type.
- Add optional cookie field to `session::sso_login*::v3` responses.
- Add support for local user erasure to `account::deactivate::v3::Request`,
  according to MSC4025 / Matrix 1.10.
- Allow `discovery::get_supported_versions::v1` to optionally accept
  authentication, according to MSC4026 / Matrix 1.10.
- Allow `account::register::v3` and `account::login::v3` to accept
  authentication for appservices.
- Add support for recursion on the `get_relating_events` endpoints, according to
  MSC3981 / Matrix 1.10
- Add server support discovery endpoint, according to MSC1929 / Matrix 1.10
- Add `dir` `Request` field on the `get_relating_events_with_rel_types` and
  `get_relating_events_with_rel_type_and_event_type` endpoints
- Add unstable support for moderator server support discovery, according to MSC4121
- Add unstable support for the room summary endpoint from MSC3266 behind the
  `unstable-msc3266` feature.
- Add unstable support for animated thumbnails, according to MSC2705

# 0.17.4

Improvements:

- Change the `avatar` field of `SlidingSyncRoom` from `Option` to `JsOption`
  - This is a breaking change, but only for users enabling the
    `unstable-msc3575` feature

# 0.17.3

Bug fixes:

- Fix deserialization of `claim_keys` responses without a `failures` field

# 0.17.2

Improvements:

- Add unstable support for MSC3983

# 0.17.1

Improvements:

- Add a ErrorKind variant for the "M_WRONG_ROOM_KEYS_VERSION" Matrix error.

# 0.17.0

Breaking changes:

- Define `rank` as an `Option<f64>` instead of an `Option<UInt>` in
  `search::search_events::v3::SearchResult`
- Remove the `token` field from `keys::get_keys::Request`, according to a spec clarification.
- `SpaceRoomJoinRule` has been moved to the `space` module of the ruma-common crate
- `backup::SessionData(Init)` were renamed to `EncryptedSessionData(Init)`

Improvements:

- Add convenience constructors for enabling lazy-loading in filters
- Add support for using an existing session to log in another (MSC3882 / Matrix 1.7)
- Add support for media download redirects (MSC3860 / Matrix 1.7)
- Stabilize support for asynchronous media uploads (MSC2246 / Matrix 1.7)
- Add support for the appservice ping mechanism (MSC 2659 / Matrix 1.7)

# 0.16.2

Bug fixes:

- Don't serialize `None` as `null` in `report_content::v3::Request`

# 0.16.1

Improvements:

* Stabilize support for getting an event by timestamp (MSC3030 / Matrix 1.6)

# 0.16.0

Breaking changes:

* Remove `sync::sync_events::v3::DeviceLists` re-export
  * Use `sync::sync_events::DeviceLists` instead
* `fully_read` field in `read_marker::set_read_marker` is no longer required
  * Remove the `fully_read` argument from `read_marker::set_read_marker::Request::new`
* Move `message::get_message_events::v3::Direction` to `ruma-common::api`
* Make `push::set_pusher::v3::Request` use an enum to differentiate when deleting a pusher
  * Move `push::get_pushers::v3::Pusher` to `push` and make it use the new `PusherIds` type
  * Remove `push::set_pusher::v3::Pusher` and use the common type instead
* Make `push::PusherKind` contain the pusher's `data`
* Use an enum for the `scope` of the `push` endpoints
* Use `NewPushRule` to construct a `push::set_pushrule::v3::Request`
* `Error` is now an enum because endpoint error construction is infallible (see changelog for
  `ruma-common`); the previous fields are in the `Standard` variant
* Use `GlobalAccountDataEventType` for `event_type` in `config::get_global_account_data`
* Use `RoomAccountDataEventType` for `event_type` in `config::get_room_account_data`
* Use `ToDeviceEventType` for `event_type` in `to_device::send_event_to_device`

Improvements:

* Add `M_BAD_ALIAS` to `error::ErrorKind`
* Remove the `unstable-msc3440` feature
  * The fields added to `RoomEventFilter` were removed by MSC3856
* Add support for the threads list API (MSC3856 / Matrix 1.4)
* Stabilize support for private read receipts
* Add support for the pagination direction parameter to `/relations` (MSC3715 / Matrix 1.4)
* Add support for notifications for threads (MSC3773 / Matrix 1.4)
* Send CORP headers by default for media responses (MSC3828 / Matrix 1.4)
* Add support for read receipts for threads (MSC3771 / Matrix 1.4)
* Add unstable support to get an event by timestamp (MSC3030)
* Add unstable support for discovering a sliding sync proxy (MSC3575)

# 0.15.3

Bug fixes:

* Don't include sensitive information in `Debug`-format of types from the `login` module

# 0.15.2

Yanked since it the minimum version for the `ruma-common` dependency was wrong.

# 0.15.1

Improvements:

* `DeviceLists` has moved from `sync::sync_events::v3` to `sync::sync_events`
  * It is still available under the old location for backwards compatibility

# 0.15.0

Breaking changes:

* Export nothing from the crate if neither the `client` nor the `server` feature is active
  * This may partially be reverted in subsequent releases
* `UnreadNotificationsCount` has moved from `sync::sync_events::v3` to `sync::sync_events`
* Remove `PartialEq` implementations for a number of types
  * If the lack of such an `impl` causes problems, please open a GitHub issue
* Split `uiaa::UserIdentifier::ThirdParty` into two separate variants
* Remove the `from` parameter from `message::get_message_events::v3::Request`'s constructors
  * This affects `new`, `backward` and `forward`
  * Since `backward` and `forward` are equivalent to `from_end` and `from_start`, those are removed
  * A new method `.from()` was added to easily set this field after initial construction
* `receipt::create_receipt` uses its own `ReceiptType`
* Reorder parameters in `{set_global_account_data, set_room_account_data}::Request::{new, new_raw}`

Improvements:

* Add support for refresh tokens (MSC2918)
* Add `ErrorKind::{UnableToAuthorizeJoin, UnableToGrantJoin}` encountered for restricted rooms
* Add support for timestamp massaging (MSC3316)
* Add support for querying relating events (MSC2675)
* Move `filter::RelationType` to `ruma_events::relations`
* Add unstable support for discovering an OpenID Connect server (MSC2965)
* Add `SpaceRoomJoinRule::KnockRestricted` (MSC3787)
* Add unstable support for private read receipts (MSC2285)
* Add unstable support for API scope restriction (MSC2967)

# 0.14.1

Improvements:

* Add `From<&UserId>` and `From<&OwnedUserId>` implementations for `UserIdentifier`
* Add `UserIdentifier::email` constructor

# 0.14.0

Bug fixes:

* Fix HTTP method of `backup::update_backup`
* Make score and reason optional in room::report_content::Request
* Fix `uiaa::*::thirdparty_id_creds` according to a clarification in the spec

Breaking changes:

* Use `Raw` for `config::set_*_account_data::Request::data`.
* Rename the endpoints in `backup`:
  * `add_backup_key_session` => `add_backup_keys_for_session`
  * `add_backup_key_sessions` => `add_backup_keys_for_room`
  * `create_backup` => `create_backup_version`
  * `delete_backup` => `delete_backup_version`
  * `delete_backup_key_session` => `delete_backup_keys_for_session`
  * `delete_backup_key_sessions` => `delete_backup_keys_for_room`
  * `get_backup` => `get_backup_info`
  * `get_backup_key_session` => `get_backup_keys_for_session`
  * `get_backup_key_sessions` => `get_backup_keys_for_room`
  * `get_latest_backup` => `get_latest_backup_info`
  * `update_backup` => `update_backup_version`
* Rename `discover` to `discovery`
* Move `capabilities::get_capabilities` into `discovery`
* Make `from` optional in `message::get_message_events` according to a clarification in the spec

Improvements:

* Add support for the space summary API in `space::get_hierarchy` according to MSC2946.
* Add `device_id` to `account::whoami::Response` according to MSC2033
* Add `is_guest` to `account::whoami::Response` according to MSC3069
* Add `session::login::LoginInfo::ApplicationService` according to MSC2778
* Add new fields in `discovery::get_capabilities::Capabilities` according to MSC3283
* Implement Space Summary API according to MSC2946
* Add unstable support for threads in `filter::RoomEventFilter` according to MSC3440

# 0.13.0

Bug fixes:

* Fix deserialization of `r0::session::get_login_types::CustomLoginType`.
* Make fields of `r0::session::get_login_types::IdentityProvider` public.

Breaking changes:

* Use an enum for user-interactive auth stage type (used to be `&str` / `String`)
* Make `r0::uiaa::ThirdpartyIdCredentials` an owned type and remove its `Incoming` equivalent
  * Previously, we had two fields of type `&'a [ThirdpartyIdCredentials<'a>]` and this kind of
    nested borrowing can be very annoying
* `LoginInfo` no longer implements `PartialEq` and `Eq` due to the custom variant that was added.
* `LoginInfo` converted to newtype variants.
* Use `Raw` for `create_room::Request::creation_content`
* Delete `r0::contact` module
  * `request_contact_verification_token` was an out-of-date duplicate of
    `r0::account::request_3pid_management_token_via_email`
  * `get_contacts` has been can now be found at `r0::account::get_3pids`
* Move `r0::uiaa::authorize_fallback` to `r0::uiaa::get_uiaa_fallback_page`
* Change type of field `start` of `r0::message::get_message_events::Response` to
  `String` in accordance with the updated specification.
* Rename `uiaa::UserIdentifier::MatrixId` variant to `uiaa::UserIdentifier::UserIdOrLocalpart`

Improvements:

* Add support for reasons in the membership endpoints:

  ```rust
  r0::membership::{
    join_room_by_id,
    join_room_by_id_or_alias,
    invite_user,
    unban_user
  }
  ```
* Add a `.data()` accessor method to `r0::uiaa::{AuthData, IncomingAuthData}`
* Allow to construct the custom `AuthData` variant with `IncomingAuthData::new` and then call
  `IncomingAuthData::to_outgoing` on it.
* Add custom variant to `LoginInfo` which can be constructed with `IncomingLoginInfo::new` and
  then call `IncomingLoginInfo::to_outgoing` on it.
* Move MSC2858 - Multiple SSO Identity Providers out of the `unstable-pre-spec` feature flag, this
  includes:
  * The `r0::session::get_login_types::{IdentityProvider, IdentityProviderBrand}` types
  * The `session::sso_login_with_provider::v3` endpoint
* Move reason support for leaving room out of `unstable-pre-spec`
* Move room type support out of `unstable-pre-spec`
* Move knocking support out of `unstable-pre-spec`
* Move blurhash support to `unstable-msc2448`

# 0.12.3

* Add a `feature = "compat"` workaround for Element failing on `GET /_matrix/client/r0/account/3pid`
  response if the optional `threepids` field is missing

# 0.12.2

Improvements

* Add `auth_type` and `session` accessors to `uiaa::IncomingAuthData`

# 0.12.1

Improvements:

* Add `auth_type` and `session` accessors to `uiaa::AuthData`

# 0.12.0

Breaking changes:

* Change inconsistent types in `rooms` and `not_rooms` fields in
  `RoomEventFilter` structure: both types now use `RoomId`
* Move `r0::{session::login::UserIdentifier => uiaa::UserIdentifier}`
* Add `stages` parameter to `r0::uiaa::AuthFlow::new`
* Upgrade dependencies

Improvements:

* Add more endpoints:

  ```rust
  r0::knock::knock_room
  ```
* Add unstable support for room knocking
* Add unstable support for reasons for leaving rooms

# 0.11.2

Yanked since it depended on a version of ruma-api that had to be yanked too.

# 0.11.1

Yanked, wrong dependency version.

# 0.11.0

Breaking changes:

* Use `Raw<AnyInitialStateEvent>` over just `AnyInitialStateEvent` in the `initial_state` field
  of `r0::room::create_room::Request`
* Remove

  ```rust
  r0::keys::{
      CrossSigningKey, CrossSigningKeySignatures, KeyUsage, OneTimeKey, SignedKey,
      SignedKeySignatures,
  }
  ```

  These are now found in `ruma_common::encryption` (or `ruma::encryption`).
* Remove `r0::to_device::DeviceIdOrAllDevices`, now found in `ruma_common::to_device`
  (or `ruma::to_device`)
* Remove `r0::contact::get_contacts::{ThirdPartyIdentifier, ThirdPartyIdentifierInit}`, now found
  in `ruma_common::thirdparty` (or `ruma::thirdparty`)

# 0.10.2

Bug fixes:

* Remove authentication for get alias endpoint

# 0.10.1

Improvements:

* Add unstable support for room types

# 0.10.0

Bug fixes:

* Fix deserialization of `r0::room::get_room_event::Response`
* More missing fields in `r0::sync::sync_events::Response` can be deserialized
* Fix `get_tags::Response` serialization
* Fix unsetting avatar URL when `compat` feature is enabled

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
* Move `user: UserInfo` in `r0::session::login::Request` to `identifier: UserIdentifier` in
  `r0::session::login::LoginInfo::Password`
  * `r0::session::login::Request::new` takes only `login_info: LoginInfo` as a param
* Change `ruma_events::AnyEvent` to `ruma_events::AnySyncRoomEvent` in
  `push::get_notifications::Notification`
* Use `ruma_identifiers::MxcUri` instead of `String` for `avatar_url` fields in the following
  endpoints:
  ```rust
  r0::{
      directory,
      media::create_content,
      membership::joined_members,
      profile::{get_avatar_url, get_profile, set_avatar_url},
      search::{search_events, search_users}
  }
  ```
* Change `r0::session::get_login_types::LoginType` to a non-exhaustive enum of structs.
* Move `r0::receipt::ReceiptType` to the `ruma-common` crate

Improvements:

* Add method `into_event_content` for `r0::room::create_room::CreationContent`
* Add room visibility endpoints: `r0::directory::{get_room_visibility, set_room_visibility}`.
* Add is_empty helpers for structs in `r0::sync::sync_events`
* Add a constructor for request structs of the following endpoints
  * `r0::room::create_room`
  * `r0::message::get_message_events`
* Add `logout_devices` field to `r0::account::change_password`
* Add `r0::room::aliases` (introduced in r0.6.1)
* Add constructors that use `ruma_identifiers::MxcUri` for `Request` in the following endpoints:
  ```rust
  r0::media::{
      get_content,
      get_content_as_filename,
      get_content_thumbnail
  }
  ```
* Implement MSC2858 - Multiple SSO Identity Providers under the `unstable-pre-spec` feature flag:
  * Add the `r0::session::get_login_types::{IdentityProvider, IdentityProviderBrand}` types
  * Add the `r0::session::sso_login_with_provider` endpoint

# 0.9.0

Bug fixes:

* Fix (de)serialization for `r0::media::get_content_thumbnail::Response`
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
* Remove dependency on the `url` crate

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
  * `r0::account::request_3pid_management_token_via_email`
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
