// Runs at the head of `Game::native_is_signal_source` (API 38): the two
// redstone glasses answer as sources.
fn observe_signal_source(
    game: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    source: &mut bool,
) {
    crate::game::glassential_state::is_signal_source(game, state, source);
}
