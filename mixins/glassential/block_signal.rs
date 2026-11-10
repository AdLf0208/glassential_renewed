// Runs at the head of `Game::native_block_signal` (API 38): the two redstone
// glasses give 15 on every face.
fn observe_block_signal(
    game: &crate::game::GameLevel<'_>,
    state: horizon_world::registry::BlockStateId,
    signal: &mut i32,
) {
    crate::game::glassential_state::block_signal(game, state, signal);
}
