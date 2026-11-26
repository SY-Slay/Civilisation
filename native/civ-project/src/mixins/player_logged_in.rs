fn civ_player_logged_in(game: &mut crate::game::Game<'_>, player: u128, name: &str) {
    crate::game::civ_glue::player_logged_in(game, player, name);
}
