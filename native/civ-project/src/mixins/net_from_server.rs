fn civ_net_from_server(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    crate::game::civ_glue::net_from_server(game, channel, data);
}
