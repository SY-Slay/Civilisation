fn civ_screen_open(game: &mut crate::game::Game<'_>, mod_id: &str, opened: &mut bool) {
    crate::game::civ_glue::screen_open(game, mod_id, opened);
}
