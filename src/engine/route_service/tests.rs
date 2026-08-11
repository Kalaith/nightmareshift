use crate::data::loader::load_constants;

/// The soft warning must land far enough ahead of the hard one to be
/// worth acting on, and both must clear the cost of a single route —
/// a warning that arrives with no affordable move left is just an
/// announcement that the shift is over.
#[test]
fn the_shift_end_warning_leaves_room_to_act() {
    let constants = load_constants();
    let timing = &constants.timing;
    assert!(
        timing.shift_end_warning_threshold > timing.critical_time_threshold,
        "warning at {} is not ahead of critical at {}",
        timing.shift_end_warning_threshold,
        timing.critical_time_threshold
    );

    let game = &constants.game_constants;
    let cheapest_route = game
        .time_cost_shortcut
        .min(game.time_cost_normal)
        .min(game.time_cost_scenic)
        .min(game.time_cost_police);
    assert!(
        timing.shift_end_warning_threshold > cheapest_route,
        "the warning at {} arrives with no route affordable (cheapest is {})",
        timing.shift_end_warning_threshold,
        cheapest_route
    );
}
