fn civ_camera_shake(
    game: &crate::game::Game<'_>,
    partial: f32,
    shake: &mut crate::game::native_hooks::NativeShake,
) {
    crate::game::civ_glue::camera_shake(game, partial, shake);
}
