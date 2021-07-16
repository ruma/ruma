# [unreleased]

Breaking changes:

* state_res::resolve auth_events type has been slightly changed and renamed to auth_chain_sets
* state_res::resolve structs were changed from BTreeMap/Set to HashMap/Set

# 0.2.0

Breaking changes:

* Replace `Vec` by `BTreeSet` in parts of the API
* Replace `event_map` argument with a closure to fetch events on demand

# 0.1.0

Initial release
