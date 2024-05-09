# [unreleased]

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
