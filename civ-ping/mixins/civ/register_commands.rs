// Adaptateur fin du hook `register_commands`.
// La logique testable vit dans le module `crate::game::civ` (runtime.rs).
fn civ_register_commands(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<crate::mod_api::NativeCommand>,
) {
    crate::game::civ::register_commands(game, commands);
}
