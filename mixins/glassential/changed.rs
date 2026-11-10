// Runs at the head of `Game::native_block_changed` and
// `Game::native_neighbor_changed`: remember the cell for the smart-glass
// panel rebuild in `level_tick_post`.
fn observe_change(
    dimension: horizon_world::level::Dimension,
    pos: horizon_core::position::BlockPos,
) {
    crate::game::glassential_state::on_changed(dimension, [pos.x, pos.y, pos.z]);
}
