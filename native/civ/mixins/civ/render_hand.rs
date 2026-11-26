fn civ_render_hand(
    game: &crate::game::Game<'_>,
    hand: horizon_client::first_person::Hand,
    stack: Option<&horizon_client::items::ItemStack>,
    partial: f32,
    swing: f32,
    equip: f32,
    canceled: &mut bool,
) {
    crate::game::civ_glue::render_hand(game, hand, stack, partial, swing, equip, canceled);
}
