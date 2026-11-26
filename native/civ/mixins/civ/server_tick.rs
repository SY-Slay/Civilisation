fn civ_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &crate::mod_api::NativeLifecycleContext,
    phase: crate::mod_api::ServerTickPhase,
) {
    crate::game::civ_glue::server_tick(game, context, phase);
}
