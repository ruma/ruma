//! Snapshot tests.

// Test the minimal set of events required to create a room with the
// "private_chat" preset.
snapshot_test!(minimal_private_chat, ["bootstrap-private-chat.json"]);
