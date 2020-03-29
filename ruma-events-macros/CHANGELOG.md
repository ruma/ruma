# [unreleased]

Breaking changes:

* Update `event_type` in `ruma_events!` to refer to the serialized form of the
  event type, not the variant of `ruma_events::EventType`

Improvements:

* Split `FromRaw` implementation generation from `ruma_event!` into a separate
  proc-macro

# 0.2.0

Improvements:

* Code generation was updated to account for the changes in ruma-events 0.15
* Dependencies were updated (notably to syn 1.0 and quote 1.0)
