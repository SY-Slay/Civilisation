fn civ_client_tick(game: &mut crate::game::Game<'_>) {
    crate::game::civ_glue::client_tick(game);
}
