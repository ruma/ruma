# [unreleased]

Breaking:

- `auth_check` returns a `Result<(), String>` instead of a
  `Result<bool, Error>`. A successful check now returns `Ok(())` instead of
  `Ok(true)` and all failures return an `Err(_)` with a description of the check
  that failed.
- The variants of `Error` were changed:
  - `Unsupported` was removed since we always take an `AuthorizationRules`
    instead of a `RoomVersionId`.
  - `NotFound` holds an `OwnedEventId`.
  - The cases that were triggering an `InvalidPdu` error now trigger a
    `MissingStateKey` error.
  - The cases that were triggering a `SerdeJson` or a `Custom` error are either
    ignored when coming from the `auth_check()` (see corresponding bug fix) or
    return an `AuthEvent` error.
- `auth_types_for_event` takes an `AuthorizationRules`, to check if restricted
  join rules are allowed before looking for the
  `join_authorised_via_users_server` field in `m.room.member`.
- `resolve` takes an `AuthorizationRules` instead of a `RoomVersionId`. This 
  allows server implementations to support custom room versions. They only need
  to provide an `AuthorizationRules` for their custom `RoomVersionId`.
- `RoomVersion` was moved to ruma-common and renamed `RoomVersionRules`, along
  with other changes. Check the changelog of ruma-common for more details.
- The `event_auth` module is no longer public. Everything public inside of it
  is already exposed at the root of the crate.
- `auth_check` was split into 2 functions: `check_state_independent_auth_rules`
  and `check_state_dependent_auth_rules`. The former should be called once when
  the incoming event is received, while the latter should be called for every
  state that should be checked.

Bug fixes:

- Don't propagate errors from `auth_check()` in `resolve()`. If an event fails
  the authorization check, it should just be ignored for the resolved state.
- Don't error on deserialization of malformed fields that are not checked in the
  authorization rules for `m.room.create`, `m.room.member`,
  `m.room.power_levels`, `m.room.join_rules` and `m.room.third_party_invite`
  events.
- Fix `auth_check` for `m.room.member` with an `invite` membership and a
  `third_party_invite`. The `signed` object in the content is now verified
  against the public keys in the matching `m.room.third_party_invite` event.

Improvements:

- New types with lazy deserialization that can be used by servers over the
  stricter types from ruma-events to access the fields of an event when received
  over federation, to avoid erroring on malformed fields that are not checked by
  the authorization rules:
  - `RoomCreateEvent` for `m.room.create` events
  - `RoomMemberEvent` for `m.room.member` events
  - `RoomPowerLevelsEvent` for `m.room.power_levels` events
  - `RoomJoinRulesEvent` for `m.room.join_rules` events
  - `RoomThirdPartyInviteEvent` for `m.room.third_party_invite` events

# 0.13.0

Bug fixes:

- Fix tiebreaking logic in state resolution.

Improvements:

- The `unstable-exhaustive-types` cargo feature was replaced by the
  `ruma_unstable_exhaustive_types` compile-time `cfg` setting. Like all `cfg`
  settings, it can be enabled at compile-time with the `RUSTFLAGS` environment
  variable, or inside `.cargo/config.toml`. It can also be enabled by setting
  the `RUMA_UNSTABLE_EXHAUSTIVE_TYPES` environment variable.

# 0.12.0

Upgrade `ruma-events` to 0.29.0.

# 0.11.0

Breaking changes:

- Upgrade dependencies

Bug fixes:

- Disallow `invite` -> `knock` membership transition.
  The spec was determined to be right about rejecting it in the first place:
  <https://github.com/matrix-org/matrix-spec/pull/1717>
- Perform extra redaction checks on room versions 1 and 2, rather than for
  version 3 and onwards

# 0.10.0

Improvements:

- Add `RoomVersion::V11` according to MSC3820 / Matrix 1.8

# 0.9.1

No changes for this version

# 0.9.0

Bug fixes:

* Fix third party invite event authorization. The event was not allowed even
  after passing all the required checks, so it could fail further down the
  algorithm.
* Allow `invite` -> `knock` membership transition
  * The spec was determined to be wrong about rejecting it:
    <https://github.com/matrix-org/matrix-spec/pull/1175>

# 0.8.0

Bug fixes:

* Change default `invite` power level to `0`
  * The spec was determined to be wrong about the default:
    <https://github.com/matrix-org/matrix-spec/pull/1021>

Improvements:

* Add `m.federate` to `auth_check`:
  <https://github.com/matrix-org/matrix-spec/pull/1103>
* Add `RoomVersion::V10` (MSC3604)
* Deserialize stringified integers for power levels without the `compat` feature
  * Removes the `compat` feature

# 0.7.0

Breaking changes:

* `auth_check` does not require `prev_event` parameter. It was only required on
  some specific cases. Previous event is now calculated on demand only when
  it's required.

# 0.6.0

Breaking changes:

* Upgrade dependencies

# 0.5.0

Breaking changes:

* Remove some trait methods from `Event`
* Update `Event::content` signature to return `&RawJsonValue` instead of `&JsonValue`
* The `key_fn` in `lexicographical_topological_sort` has removed the event ID from its return type
  and changed to expect just the power level, not the negated power level

# 0.4.1

Improvements:

* Improve performance of `StateResolution::separate`

# 0.4.0

Breaking changes:

* Change the way events are supplied

# 0.3.0

Breaking changes:

* state_res::resolve auth_events type has been slightly changed and renamed to auth_chain_sets
* state_res::resolve structs were changed from BTreeMap/Set to HashMap/Set
* Upgrade dependencies

# 0.2.0

Breaking changes:

* Replace `Vec` by `BTreeSet` in parts of the API
* Replace `event_map` argument with a closure to fetch events on demand

# 0.1.0

Initial release
