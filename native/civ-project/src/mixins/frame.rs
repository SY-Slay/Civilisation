fn civ_frame(game: &mut crate::game::Game<'_>, elapsed: std::time::Duration) {
    crate::game::civ_glue::frame(game, elapsed);
}
