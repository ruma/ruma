//! State resolution integration tests.

#[macro_use]
mod macros;

// Test the minimal set of events required to create a room with the
// "private_chat" preset.
snapshot_test_batches!(minimal_private_chat, ["bootstrap-private-chat.json"]);

// A create a room with the "public_chat" preset and bob as an admin.
snapshot_test_batches!(minimal_public_chat, ["bootstrap-public-chat.json"]);

// Start with a private room, then transition its join rules to restricted, then
// to public. The events in the second file are tied topologically, so they must
// have the tiebreaking algorithm applied. The ordering should be decided by
// the `origin_server_ts` fields of these events, not the `event_id` fields. The
// power levels of these events are equivalent, so they don't really matter.
snapshot_test_batches!(
    origin_server_ts_tiebreak,
    ["bootstrap-private-chat.json", "origin-server-ts-tiebreak.json"],
);

// Test that state res v2.0 is implemented starting from the unconflicted set, and NOT the empty
// set, leading to there being no join rules state.
//
// This example comes directly from the "Problem A" section of MSC4297.
snapshot_test_state_maps!(
    msc4297_problem_a_state_res_v2_0,
    ["MSC4297-problem-A/state-bob.json", "MSC4297-problem-A/state-charlie.json"],
    ["MSC4297-problem-A/pdus-v11.json"],
);

// Test that state res v2.1 is implemented starting from the empty set, and NOT the unconflicted
// set.
//
// This example comes directly from the "Problem A" section of MSC4297.
snapshot_test_state_maps!(
    msc4297_problem_a_state_res_v2_1,
    ["MSC4297-problem-A/state-bob.json", "MSC4297-problem-A/state-charlie.json"],
    ["MSC4297-problem-A/pdus-v12.json"],
);

// Test that state res v2.0 does NOT consider the conflicted state subgraph as part of the full
// conflicted state set, leading to the state resetting to the first power levels event.
//
// This example comes directly from the "Problem B" section of MSC4297.
snapshot_test_state_maps!(
    msc4297_problem_b_state_res_v2_0,
    ["MSC4297-problem-B/state-eve.json", "MSC4297-problem-B/state-zara.json"],
    ["MSC4297-problem-B/pdus-v11.json"],
);

// Test that state res v2.1 considers the conflicted state subgraph as part of the full conflicted
// state set.
//
// This example comes directly from the "Problem B" section of MSC4297.
snapshot_test_state_maps!(
    msc4297_problem_b_state_res_v2_1,
    ["MSC4297-problem-B/state-eve.json", "MSC4297-problem-B/state-zara.json"],
    ["MSC4297-problem-B/pdus-v12.json"],
);

// Test that a power levels change from bob is superseded by his ban from alice.
snapshot_test_batches!(
    ban_vs_power_levels,
    [
        "bootstrap-public-chat.json",
        "ban-vs-power-levels-alice.json",
        "ban-vs-power-levels-bob.json",
    ],
);

// Test that a room topic change from bob is superseded by his demotion from alice.
snapshot_test_batches!(
    topic_vs_power_levels,
    [
        "bootstrap-public-chat.json",
        "topic-vs-power-levels-alice.json",
        "topic-vs-power-levels-bob.json",
    ],
);

// Test that a power levels change from bob is superseded by his demotion from alice.
snapshot_test_batches!(
    power_levels_admin_vs_mod,
    [
        "bootstrap-public-chat.json",
        "power-levels-admin-vs-mod-alice.json",
        "power-levels-admin-vs-mod-bob.json",
    ],
);

// Test that a room topic change from bob is superseded by his ban from alice.
snapshot_test_batches!(
    topic_vs_ban,
    [
        "bootstrap-public-chat.json",
        "topic-vs-ban-common.json",
        "topic-vs-ban-alice.json",
        "topic-vs-ban-bob.json",
    ],
);

// Test that a join from ella is superseded by a join rules change from alice.
snapshot_test_batches!(
    join_rules_vs_join,
    [
        "bootstrap-public-chat.json",
        "join-rules-vs-join-common.json",
        "join-rules-vs-join-alice.json",
        "join-rules-vs-join-ella.json",
    ],
);

// Test that concurrent joins both end up in the state.
snapshot_test_batches!(
    concurrent_joins,
    ["bootstrap-public-chat.json", "concurrent-joins-charlie.json", "concurrent-joins-ella.json",],
);
