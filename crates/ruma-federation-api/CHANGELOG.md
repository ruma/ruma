# [unreleased]

Breaking Changes:

* Change types of keys::claim_keys::v1 response to match the client-server endpoint

Improvements:

* Add master_keys and self_signing keys to keys::get_keys::v1 response

# 0.1.0

Breaking Changes:

* Replace `directory::get_public_rooms::v1::{PublicRoomsChunk, RoomNetwork}` with types from
  `ruma_common::directory`
* Wrap `PduStub`s in `membership::create_join_event` in `Raw`
* Remove `PduStub` (it only existed because of the spec being misleading)
  * Rename `pdu_stub` fields to `pdu`
* Upgrade dependencies
* Wrap `Pdu`s in `backfill::get_backfill` in `Raw`
* Use `ruma_identifiers::MxcUri` instead of `String` for `avatar_url` in
  `query::get_profile_information::v1`
* Rename `homeserver` property to `server` on `discover_homeserver::Response`

Improvements:

* Add endpoints:

  ```
  backfill::get_backfill::v1,
  device::get_devices::v1,
  directory::get_public_rooms_filtered::v1,
  event::get_missing_events::v1,
  keys::{
      claim_keys::v1,
      query_keys::v1,
  },
  membership::{
      create_invite::{v1, v2},
      create_join_event::v2,
      create_leave_event::{v1, v2},
      get_leave_event::v1,
  },
  query::get_custom_information::v1,
  thirdparty::{
      bind_callback::v1,
      exchange_invite::v1,
  },
  ```

Bug fixes:

* Fixes `discover_homeserver::Response` serialization and deserialization

# 0.0.3

Breaking Changes:

* Replace `RoomV3Pdu` with `ruma_events::pdu::{Pdu, PduStub}`.

Improvements:

* Add endpoints:

  ```
  authorization::get_event_authorization::v1,
  openid::get_openid_userinfo::v1,
  query::get_profile_information::v1,
  transactions::send_transaction_message::v1,
  ```

# 0.0.2

Improvements:

* Add endpoints:

  ```
  directory::get_public_rooms::v1,
  discovery::{
      discover_homeserver,
      get_server_keys::v2,
      get_server_version::v1
  },
  membership::{
      create_join_event::v1,
      create_join_event_template::v1
  },
  query::get_room_information::v1,
  version::get_server_version::v1
  ```

# 0.0.1

Improvements:

* Provide `RoomV3Pdu` type for room versions 3 and above
