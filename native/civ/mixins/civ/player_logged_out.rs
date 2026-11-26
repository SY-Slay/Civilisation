fn civ_player_logged_out(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    crate::game::civ_glue::player_logged_out(game, player, name);
}
