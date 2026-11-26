fn civ_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::civ_glue::key(game, key, pressed);
}
