// Civilisations — étape 0 : noyau minimal du mod natif `civ`.
// Ce fichier est injecté comme module `crate::game::civ` par horizon-mixins.
// Contraintes du format : pas de commentaire interne `//!`, pas de `mod`
// externe, pas de `include!`. Le code peut utiliser l'API interne de Horizon.

/// Identifiant du bundle. Il doit être identique dans `horizon-mixin.toml`
/// et dans `horizon-native.json`.
pub(crate) const ID: &str = "civ";

/// Reconnaît la seule commande de l'étape 0 : `civ ping`.
/// Le slash initial et les espaces multiples sont tolérés.
pub(crate) fn is_ping_command(line: &str) -> bool {
    let mut words = line.trim().trim_start_matches('/').split_whitespace();
    words.next() == Some("civ") && words.next() == Some("ping") && words.next().is_none()
}

/// Enregistre `/civ ping` dans l'arbre de commandes du serveur.
/// Horizon appelle ce hook quand un monde s'ouvre aux joueurs, puis après
/// `native_refresh_commands`.
pub(crate) fn register_commands(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<crate::mod_api::NativeCommand>,
) {
    if game.native_on(ID) {
        commands.push(crate::mod_api::NativeCommand::new("civ ping"));
    }
}

/// Répond à `/civ ping` quand un joueur la tape.
/// `native_feedback` envoie la ligne au joueur dont la commande est en cours
/// d'exécution ; le serveur a déjà vérifié son niveau de permission.
pub(crate) fn perform_command(
    game: &mut crate::game::Game<'_>,
    command: &str,
    ran: &mut bool,
) {
    if *ran || !game.native_on(ID) || !is_ping_command(command) {
        return;
    }

    game.native_feedback(horizon_client::text::Text::literal("Civ: pong"));
    *ran = true;
}

#[cfg(test)]
mod tests {
    use super::is_ping_command;

    #[test]
    fn accepts_the_command_with_optional_slash_and_spaces() {
        assert!(is_ping_command("civ ping"));
        assert!(is_ping_command("/civ ping"));
        assert!(is_ping_command("  civ   ping  "));
    }

    #[test]
    fn rejects_everything_else() {
        assert!(!is_ping_command("civ"));
        assert!(!is_ping_command("civ ping now"));
        assert!(!is_ping_command("civ pong"));
        assert!(!is_ping_command("other ping"));
    }
}
