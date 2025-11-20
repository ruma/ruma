# [unreleased]

# 0.13.0

Breaking changes:

- All the endpoints use a `SinglePath` rather than a `VersionHistory` as
  `Metadata::PathBuilder`. Making a request doesn't require to provide a dummy
  `SupportedVersions` anymore.
  - The endpoints that became stable from Matrix 1.1 onwards have a new
    `unstable` module to be able to support the unstable endpoint defined in
    their MSC.

Improvements:

- `RawStrippedState::Stripped` is deprecated in favor of the `Pdu` variant,
  according to Matrix 1.16.
- `ServerSignatures` supports adding the `X-Matrix` header to outgoing requests.
- `XMatrix::request_object()` allows to construct the canonical JSON object to
  sign from a request.
- `XMatrix` can be constructed from a request with `try_from_http_request()`.
- The signature in the `sig` field of `XMatrix` can be used to verify a request
  with `verify_request()`.

# 0.12.0

Breaking changes:

- Remove the `origin` field in `create_join_event::{v1/v2}::RoomState` due to a
  clarification in the spec.
- The type of `signed` in `thirdparty::bind_callback::v1::Request` was fixed. It
  uses `Raw<SignedContent>` from `RoomMemberEventContent`.
- The type of `content` in `thirdparty::exchange_invite::v1::Request` was fixed.
  It is a `RoomMemberEventContent`. A new constructor was added,
  `with_third_party_invite()` that constructs the event content from a
  `ThirdPartyInvite`.
- Update the endpoint metadata definitions to use the new syntax for variables.
- Use `ruma_common::RoomSummary` for the `space::get_hierarchy` endpoint.
  - `SpaceHierarchyParentSummary` is now built around `RoomSummary`, and
    `SpaceHierarchyParentSummaryInit` was removed.
  - `SpaceHierarchyChildSummary` was replaced by `RoomSummary` and
    `SpaceHierarchyChildSummaryInit` was removed.
- Merge the `knock` module into `membership`, and rename `create_knock_event_template` and
  `send_knock` to `prepare_knock_event` and `create_knock_event` respectively for consistency.
- Use `RawStrippedState` instead of `Raw<AnyStrippedStateEvent>`, to allow non-stripped events to be
  represented for `create_invite` and `create_knock_event`.

Bug fixes:

- Add constructor for `thirdparty::bind_callback::v1::Response`.

Improvements:

- ruma-server-util was merged into this crate. `XMatrix` is available in the
  `authentication` module.
- Add a method to construct a `thirdparty::exchange_invite::v1::Request` from a
  `thirdparty::bind_callback::v1::ThirdPartyInvite` and a
  `RoomThirdPartyInviteEventContent`.
- Add unstable support for full PDUs in `create_invite` and `create_knock_event` alongside stripped
  events from MSC4311 behind the `unstable-msc4311` feature.

# 0.11.2

Bug fixes:

- Restore the `unstable-unspecified` cargo feature. There are still a few places
  that rely on that feature that would require a breaking change to remove the
  feature.

Improvements:

- Add the `encryption` and `room_version` fields to
  `SpaceHierarchyParentSummary` and `SpaceHierarchyChildSummary`, according to
  MSC3266 / Matrix 1.15.

# 0.11.1

Improvements:

- The `unstable-unspecified` cargo feature was removed. The `pdus` field of
  `send_transaction_message::v1::Request` is always serialized. To allow this
  field to be missing during deserialization, use the `compat-optional-txn-pdus`
  cargo feature.

# 0.11.0

Improvements:

- The `unstable-exhaustive-types` cargo feature was replaced by the
  `ruma_unstable_exhaustive_types` compile-time `cfg` setting. Like all `cfg`
  settings, it can be enabled at compile-time with the `RUSTFLAGS` environment
  variable, or inside `.cargo/config.toml`. It can also be enabled by setting
  the `RUMA_UNSTABLE_EXHAUSTIVE_TYPES` environment variable.

# 0.10.0

Breaking changes:

- Remove the unused `KeyObject` struct. It is actually supposed to be the same type
  as `ruma_common::encryption::SignedKey`.
- Use `OwnedOneTimeKeyId` and `OneTimeKeyAlgorithm` instead of
  `OwnedDeviceKeyId` and `DeviceKeyAlgorithm` respectively to identify one-time
  and fallback keys and their algorithm.
- Use `ServerSignatures` for the `signatures` or `ServerSigningKeys`.

Bug fixes:

- `ServerSigningKeys` can be deserialized when `old_verify_keys` is missing, due to a
  clarification in the spec.

Improvements:

- Add support for authenticated media endpoints, according to MSC3916 / Matrix 1.11
- Make `Content-Type` and `Content-Disposition` mandatory when creating media
  responses, according to MSC2701 / MSC2702 / Matrix 1.12.

# 0.9.0

Breaking changes:

- Use `RawValue` to represent body of `/v1/send_join` request, rather than incorrectly using
  query parameters
- The http crate had a major version bump to version 1.1

Improvements:

- Implement `From<SpaceHierarchyParentSummary>` for `SpaceHierarchyChildSummary`
- Add unstable support for optional `via` field on the `create_invite` endpoint request from
  MSC4125 behind the `unstable-msc4125` feature.
- Add unstable support for the `report_content` endpoint from MSC3843 behind the
  `unstable-msc3843` feature.

# 0.8.0

Bug fixes:

* Use `SpaceRoomJoinRule` for `SpaceHierarchy(Parent/Child)Summary(Init)`. Even if
  (de)serialization worked before, it is more correct to expect any join rule, like in the CS API

Improvements:

* Deprecate the `v1/send_join` and `v1/send_leave` endpoints according to a spec clarification

# 0.7.1

Improvements:

* Stabilize support for getting an event by timestamp (MSC3030 / Matrix 1.6)
* Stabilize support for partial state in `v2/send_join` (MSC3706 / Matrix 1.6)

# 0.7.0

Bug fixes:

* Add the `event` field to `RoomState` according to MSC3083 / Matrix v1.2

Breaking changes:

* Split `membership::create_join_event::RoomState` into separate types in the `v1` and `v2` modules

Improvements:

* Add unstable support to get an event by timestamp (MSC3030)
* Add unstable support to request partial state in `send_join` (MSC3706)

# 0.6.0

Breaking changes:

* Upgrade dependencies

# 0.5.0

Improvements:

* Add support for the space summary API in `space::get_hierarchy` according to MSC2946.
* Add `transactions::edu::Edu::SigningKeyUpdate` according to MSC1756
* Add Add cross-signing fields to `get_devices::Response` according to MSC1756
* Add unstable endpoint `discovery::get_server_versions` according to MSC3723

# 0.4.0

Breaking changes:

* Replace `Raw<Pdu>` with `Box<RawJsonValue>` or `&RawJsonValue`
* Borrow more request fields
* Make `device_display_name` field optional in `DeviceListUpdateContent` and update constructor accordingly
* Remove unneeded `minimum_valid_until_ts` query parameter from `get_remote_server_keys_batch` endpoint

Improvements:

* Move `knock` module out of `unstable-pre-spec`
  * `knock:::send_knock::v1::Request` requires a PDU instead of the `knock_event`
* Move cross-signing properties of `keys::get_keys::v1::Response` out of `unstable-pre-spec`
* Move MSC implementations from `unstable-pre-spec` to per-msc features:

  ```
  unstable-msc2448
  unstable-msc3618
  ```

# 0.3.1

Bug fixes:

* Fix JSON body (de)serialization of `discovery::get_remote_server_keys::batch::v2::Request`
* Fix query parameter deserialization of `discovery::get_remote_server_keys::batch::v2::Request`

# 0.3.0

Breaking changes:

* Upgrade dependencies

Improvements:

* Add more endpoints:

  ```rust
  knock::{
    create_knock_event::v1,
    send_knock::v1,
  }
  ```

  * Add unstable support for room knocking.

# 0.2.0

Breaking Changes:

* Change types of keys::claim_keys::v1 response to match the client-server endpoint
* Update `thirdparty::bind_callback::v1::Request::new` to have a `medium` parameter

Improvements:

* Add master_keys and self_signing keys to keys::get_keys::v1 response
* Add `thirdparty::bind_callback::v1::Request::email` convenience constructor

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
