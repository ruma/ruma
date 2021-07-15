# [unreleased]

Breaking changes:

* state_res::resolve now doesn't take auth_events anymore and calculates it on its own instead

# 0.2.0

Breaking changes:

* Replace `Vec` by `BTreeSet` in parts of the API
* Replace `event_map` argument with a closure to fetch events on demand

# 0.1.0

Initial release
