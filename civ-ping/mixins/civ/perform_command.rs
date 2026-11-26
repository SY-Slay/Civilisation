// Adaptateur fin du hook `perform_command`.
// `command` arrive sans slash ; `ran` dit à Horizon que la commande est prise.
fn civ_perform_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    crate::game::civ::perform_command(game, command, ran);
}
