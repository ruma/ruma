# [unreleased]

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
