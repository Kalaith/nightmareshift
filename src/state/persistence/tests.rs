/// Every platform-conditional branch must produce something a player can
/// tell apart from the next one.
///
/// The web build has no wall clock, and the leaderboard's date column
/// fell back to the literal "Session" for every entry — ten identical
/// labels on a screen whose only job is distinguishing ten runs. The
/// wasm branch has to derive its label from something that changes.
///
/// This reads the source, so it breaks when the code moves — which it
/// did, when the shift lifecycle came out of `game.rs`. That is the
/// honest cost of the technique and preferable to the alternative, since
/// the branch cannot be evaluated on this target at all.
#[test]
fn the_web_leaderboard_label_varies_between_runs() {
    let source = include_str!("../../game/shift.rs");
    let branch = source
        .split(r#"#[cfg(target_arch = "wasm32")]"#)
        .find(|chunk| chunk.trim_start().starts_with("let date_str"))
        .expect("the wasm leaderboard label still exists");
    let branch = &branch[..branch.find(';').unwrap_or(branch.len())];

    assert!(
        branch.contains("total_shifts_completed"),
        "the web leaderboard label does not vary between runs: {branch:?}"
    );
}

/// Saving must be wired on both targets. The web build routes through the
/// toolkit's slot API and the desktop build through a file; losing either
/// silently drops the meta-progression the whole game accrues.
#[test]
fn both_targets_have_a_save_path() {
    let source = include_str!("../persistence.rs");
    for symbol in [
        "save_json",
        "load_json",
        "save_to_slot",
        "load_from_slot",
        "slot_exists",
        "delete_slot",
    ] {
        assert!(
            source.contains(symbol),
            "persistence no longer references {symbol}"
        );
    }
}
