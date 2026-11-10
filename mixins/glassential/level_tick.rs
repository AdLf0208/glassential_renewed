// Runs at the head of `Game::native_level_tick_post`: rebuild the dirty
// smart-glass panels of this level.
fn observe_level_tick(game: &mut crate::game::Game<'_>, dimension: &str) {
    crate::game::glassential_state::on_level_tick_post(game, dimension);
}
