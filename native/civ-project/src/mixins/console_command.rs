fn civ_console_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    crate::game::civ_glue::console_command(game, command, ran);
}
