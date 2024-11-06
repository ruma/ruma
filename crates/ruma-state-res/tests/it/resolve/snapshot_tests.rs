//! Snapshot tests.

// Test the minimal set of events required to create a room with the
// "private_chat" preset.
snapshot_test!(minimal_private_chat, ["bootstrap-private-chat.json"]);

// Start with a private room, then transition its join rules to restricted, then
// to public. The events in the second file are tied topologically, so they must
// have the tiebreaking algorithm applied. The ordering should be decided by
// the `origin_server_ts` fields of these events, not the `event_id` fields. The
// power levels of these events are equivalent, so they don't really matter.
snapshot_test!(
    origin_server_ts_tiebreak,
    ["bootstrap-private-chat.json", "origin-server-ts-tiebreak.json"],
);
