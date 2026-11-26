fn civ_mods(
    game: &mut crate::game::Game<'_>,
    mods: &mut Vec<crate::game::native_hooks::NativeMod>,
) {
    crate::game::civ_glue::mods(game, mods);
}
