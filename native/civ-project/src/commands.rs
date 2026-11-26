//! Commandes texte indépendantes du client.

use thiserror::Error;

use crate::civ_core::diplomacy::TreatyKind;
use crate::civ_core::error::{CoreError, CoreResult};
use crate::civ_core::ids::{
    EventChoiceId, IdError, MissileTypeId, NationId, ProvinceId, TechnologyId, WarId,
};

use crate::host::Host;
use crate::state::Session;

/// Erreur de syntaxe d'une commande.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CommandError {
    #[error("commande vide")]
    Empty,
    #[error("commande inconnue : {0}")]
    Unknown(String),
    #[error("argument manquant : {0}")]
    Missing(&'static str),
    #[error("argument inattendu : {0}")]
    UnexpectedArgument(String),
    #[error("nombre invalide : {0}")]
    InvalidNumber(String),
    #[error("identifiant invalide : {0}")]
    InvalidId(#[from] IdError),
}

/// Usages enregistrés auprès de l'API native Horizon.
///
/// Les arguments utilisent explicitement `<name:kind>` ou `[name:kind]`.
/// Cela évite les placeholders de documentation `<ticks>` qui sont invalides
/// pour le parser natif d'Horizon.
pub fn native_command_usages() -> &'static [&'static str] {
    &[
        "civ ping",
        "civ status",
        "civ tick <ticks:integer>",
        "civ save [name:word]",
        "civ player <nation:word>",
        "civ nations",
        "civ nation <nation:word>",
        "civ diplo <a:word> <b:word>",
        "civ war <a:word> <b:word> [province:word]",
        "civ peace <war:word>",
        "civ treaty <kind:word> <a:word> <b:word>",
        "civ missile <nation:word> <type:word> <province:word>",
        "civ research <nation:word> <tech:word>",
        "civ event <choice:word>",
        "civ revolution <nation:word>",
        "civ help",
    ]
}

/// Commande interprétée.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Ping,
    Status,
    Tick {
        ticks: u64,
    },
    Save {
        name: String,
    },
    Player {
        nation: NationId,
    },
    Nations,
    Nation {
        nation: NationId,
    },
    Diplomacy {
        first: NationId,
        second: NationId,
    },
    War {
        attacker: NationId,
        defender: NationId,
        province: Option<ProvinceId>,
    },
    Peace {
        war: WarId,
    },
    Treaty {
        kind: TreatyKind,
        first: NationId,
        second: NationId,
    },
    Missile {
        nation: NationId,
        missile_type: MissileTypeId,
        province: ProvinceId,
    },
    Research {
        nation: NationId,
        technology: TechnologyId,
    },
    Event {
        choice: EventChoiceId,
    },
    Revolution {
        nation: NationId,
    },
    Help,
}

impl Command {
    /// Analyse une ligne de commande, avec ou sans préfixe `/civ`.
    pub fn parse(input: &str) -> Result<Self, CommandError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(CommandError::Empty);
        }
        let without_slash = trimmed.strip_prefix('/').unwrap_or(trimmed);
        let mut parts = without_slash.split_whitespace();
        let first = parts.next().ok_or(CommandError::Empty)?;
        let first = first.to_ascii_lowercase();
        let (name, rest): (String, Vec<&str>) = if first == "civ" {
            let command = parts.next().ok_or(CommandError::Missing("commande"))?;
            (command.to_ascii_lowercase(), parts.collect())
        } else {
            (first, parts.collect())
        };

        match name.as_str() {
            "ping" => {
                reject_extra(&rest)?;
                Ok(Self::Ping)
            }
            "status" => {
                reject_extra(&rest)?;
                Ok(Self::Status)
            }
            "help" => {
                reject_extra(&rest)?;
                Ok(Self::Help)
            }
            "tick" => {
                let value = rest
                    .first()
                    .ok_or(CommandError::Missing("nombre de ticks"))?;
                if let Some(extra) = rest.get(1) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                let ticks = value
                    .parse::<u64>()
                    .map_err(|_| CommandError::InvalidNumber((*value).to_owned()))?;
                if ticks > 10_000_000 {
                    return Err(CommandError::InvalidNumber(format!(
                        "{ticks} dépasse la limite de 10 000 000"
                    )));
                }
                Ok(Self::Tick { ticks })
            }
            "save" => {
                let name = if rest.is_empty() {
                    "civilisations".to_owned()
                } else {
                    rest.join(" ")
                };
                Ok(Self::Save { name })
            }
            "player" => {
                let value = rest
                    .first()
                    .ok_or(CommandError::Missing("identifiant de nation"))?;
                if let Some(extra) = rest.get(1) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                let nation = NationId::new(*value)?;
                Ok(Self::Player { nation })
            }
            "nations" => {
                reject_extra(&rest)?;
                Ok(Self::Nations)
            }
            "nation" => {
                let value = rest
                    .first()
                    .ok_or(CommandError::Missing("identifiant de nation"))?;
                if let Some(extra) = rest.get(1) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                Ok(Self::Nation {
                    nation: NationId::new(*value)?,
                })
            }
            "diplo" | "diplomacy" => {
                let first = rest
                    .first()
                    .ok_or(CommandError::Missing("première nation"))?;
                let second = rest.get(1).ok_or(CommandError::Missing("seconde nation"))?;
                if let Some(extra) = rest.get(2) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                Ok(Self::Diplomacy {
                    first: NationId::new(*first)?,
                    second: NationId::new(*second)?,
                })
            }
            "war" => {
                let attacker = rest.first().ok_or(CommandError::Missing("attaquant"))?;
                let defender = rest.get(1).ok_or(CommandError::Missing("défenseur"))?;
                let province = rest
                    .get(2)
                    .map(|value| ProvinceId::new(*value))
                    .transpose()?;
                if let Some(extra) = rest.get(3) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                Ok(Self::War {
                    attacker: NationId::new(*attacker)?,
                    defender: NationId::new(*defender)?,
                    province,
                })
            }
            "peace" => {
                let value = rest.first().ok_or(CommandError::Missing("guerre"))?;
                if let Some(extra) = rest.get(1) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                Ok(Self::Peace {
                    war: WarId::new(*value)?,
                })
            }
            "treaty" => {
                let kind = rest
                    .first()
                    .ok_or(CommandError::Missing("type de traité"))?;
                let first = rest
                    .get(1)
                    .ok_or(CommandError::Missing("première nation"))?;
                let second = rest.get(2).ok_or(CommandError::Missing("seconde nation"))?;
                if let Some(extra) = rest.get(3) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                Ok(Self::Treaty {
                    kind: parse_treaty_kind(kind)?,
                    first: NationId::new(*first)?,
                    second: NationId::new(*second)?,
                })
            }
            "missile" => {
                let nation = rest.first().ok_or(CommandError::Missing("nation"))?;
                let missile_type = rest
                    .get(1)
                    .ok_or(CommandError::Missing("type de missile"))?;
                let province = rest.get(2).ok_or(CommandError::Missing("province cible"))?;
                if let Some(extra) = rest.get(3) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                Ok(Self::Missile {
                    nation: NationId::new(*nation)?,
                    missile_type: MissileTypeId::new(*missile_type)?,
                    province: ProvinceId::new(*province)?,
                })
            }
            "research" | "tech" => {
                let nation = rest.first().ok_or(CommandError::Missing("nation"))?;
                let technology = rest.get(1).ok_or(CommandError::Missing("technologie"))?;
                if let Some(extra) = rest.get(2) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                Ok(Self::Research {
                    nation: NationId::new(*nation)?,
                    technology: TechnologyId::new(*technology)?,
                })
            }
            "event" => {
                let choice = rest.first().ok_or(CommandError::Missing("choix"))?;
                if let Some(extra) = rest.get(1) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                Ok(Self::Event {
                    choice: EventChoiceId::new(*choice)?,
                })
            }
            "revolution" => {
                let nation = rest.first().ok_or(CommandError::Missing("nation"))?;
                if let Some(extra) = rest.get(1) {
                    return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
                }
                Ok(Self::Revolution {
                    nation: NationId::new(*nation)?,
                })
            }
            other => Err(CommandError::Unknown(other.to_owned())),
        }
    }

    /// Lignes d'aide affichables.
    pub fn help_lines() -> Vec<String> {
        vec![
            "Commandes Civilisations :".to_owned(),
            "  /civ ping — test déterministe du pont".to_owned(),
            "  /civ status — date, tick et nombre d'entités".to_owned(),
            "  /civ tick <n> — avance la simulation de n ticks".to_owned(),
            "  /civ save [nom] — scelle et écrit une sauvegarde".to_owned(),
            "  /civ player <nation> — change la nation du joueur".to_owned(),
            "  /civ nations — liste les nations".to_owned(),
            "  /civ nation <nation> — fiche détaillée".to_owned(),
            "  /civ diplo <a> <b> — relation bilatérale".to_owned(),
            "  /civ war <a> <b> [province] — déclare une guerre".to_owned(),
            "  /civ peace <guerre> — signe la paix".to_owned(),
            "  /civ treaty <type> <a> <b> — signe un traité".to_owned(),
            "  /civ missile <nation> <type> <province> — tire un missile".to_owned(),
            "  /civ research <nation> <technologie> — choisit une recherche".to_owned(),
            "  /civ event <choix> — résout l'événement actif du joueur".to_owned(),
            "  /civ revolution <nation> — force une révolution (test)".to_owned(),
            "  /civ help — affiche cette aide".to_owned(),
        ]
    }
}

fn reject_extra(rest: &[&str]) -> Result<(), CommandError> {
    if let Some(extra) = rest.first() {
        return Err(CommandError::UnexpectedArgument((*extra).to_owned()));
    }
    Ok(())
}

fn parse_treaty_kind(value: &str) -> Result<TreatyKind, CommandError> {
    match value.to_ascii_lowercase().as_str() {
        "peace" | "paix" => Ok(TreatyKind::Peace),
        "alliance" => Ok(TreatyKind::Alliance),
        "non_aggression" | "non-agression" | "nap" => Ok(TreatyKind::NonAggression),
        "trade" | "commerce" => Ok(TreatyKind::Trade),
        "defensive" | "defensif" => Ok(TreatyKind::Defensive),
        "ceasefire" | "cessez-le-feu" => Ok(TreatyKind::Ceasefire),
        "customs_union" | "douane" => Ok(TreatyKind::CustomsUnion),
        "research" | "recherche" => Ok(TreatyKind::Research),
        other => Err(CommandError::Unknown(format!("traité {other}"))),
    }
}

/// Exécute une commande déjà analysée.
pub fn execute(
    session: &mut Session,
    host: &mut dyn Host,
    command: &Command,
) -> CoreResult<Vec<String>> {
    let lines = match command {
        Command::Ping => Ok::<Vec<String>, CoreError>(vec![
            "Civilisations : pong ? d?terminisme actif".to_owned(),
        ]),
        Command::Status => {
            let status = session.status();
            let player = status
                .player_nation
                .as_ref()
                .map(|nation| nation.to_string())
                .unwrap_or_else(|| "aucune".to_owned());
            Ok(vec![
                format!("Date : {}", status.date),
                format!("Tick : {}", status.tick),
                format!(
                    "Provinces : {} · nations : {} · guerres : {} · révolutions : {}",
                    status.provinces, status.nations, status.wars, status.revolutions
                ),
                format!("Nation du joueur : {player}"),
            ])
        }
        Command::Tick { ticks } => {
            let events = session.advance_ticks(*ticks)?;
            let status = session.status();
            let mut lines = vec![format!(
                "Simulation avancée de {ticks} ticks → {} · {} événement(s)",
                status.date,
                events.len()
            )];
            for event in events.iter().rev().take(5).rev() {
                lines.push(format!("  {}", event.message));
            }
            Ok(lines)
        }
        Command::Save { name } => {
            session.save_to_host(host, name)?;
            Ok(vec![format!("Sauvegarde scellée : {name}")])
        }
        Command::Player { nation } => {
            session.set_player_nation(nation.clone())?;
            Ok(vec![format!("Nation du joueur : {nation}")])
        }
        Command::Nations => {
            let mut lines = vec!["Nations :".to_owned()];
            let mut nations: Vec<_> = session.save().nations.iter().collect();
            nations.sort_by(|left, right| left.id.cmp(&right.id));
            for nation in nations {
                lines.push(format!(
                    "  {} — {} · {} province(s) · stabilité {:.0} · PIB {:.0}",
                    nation.id,
                    nation.display_short_name(),
                    nation.province_count(),
                    nation.stability,
                    nation.economy.gdp
                ));
            }
            Ok(lines)
        }
        Command::Nation { nation } => {
            let value = session
                .nation(nation)
                .ok_or_else(|| CoreError::data(format!("nation inconnue : {nation}")))?;
            Ok(vec![
                format!("Nation : {} ({})", value.display_short_name(), value.id),
                format!(
                    "Capitale : {} · provinces : {} · population : {}",
                    value.capital,
                    value.province_count(),
                    value.population.total
                ),
                format!(
                    "Stabilité {:.1} · légitimité {:.1} · troubles {:.1} · prestige {:.1}",
                    value.stability, value.legitimacy, value.unrest, value.prestige
                ),
                format!(
                    "PIB {:.0} · trésorerie {:.0} · dette {:.0} · chômage {:.1}%",
                    value.economy.gdp,
                    value.economy.treasury,
                    value.economy.debt,
                    value.economy.unemployment * 100.0
                ),
                format!(
                    "Armée {} · main-d'œuvre {} · préparation {:.1} · moral {:.1}",
                    value.military.army_size,
                    value.military.manpower,
                    value.military.readiness,
                    value.military.morale
                ),
                format!(
                    "Recherche : {} · technologies : {} · guerres : {} · révolutions : {}",
                    value
                        .technology
                        .researching
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "aucune".to_owned()),
                    value.technology.researched.len(),
                    value.wars.len(),
                    value.revolutions.len()
                ),
            ])
        }
        Command::Diplomacy { first, second } => {
            let relation = session
                .save()
                .diplomacy
                .relation(first, second)
                .ok_or_else(|| CoreError::data("relation inconnue"))?;
            Ok(vec![
                format!("Relation {} / {}", first, second),
                format!(
                    "Opinion {:.1} · confiance {:.1} · peur {:.1} · tension {:.1}",
                    relation.opinion, relation.trust, relation.fear, relation.border_tension
                ),
                format!(
                    "Statut : {:?} · ambassade : {}",
                    relation.status, relation.embassy
                ),
            ])
        }
        Command::War {
            attacker,
            defender,
            province,
        } => {
            let war = session.declare_war(attacker, defender, province.as_ref())?;
            Ok(vec![format!("Guerre déclarée : {war}")])
        }
        Command::Peace { war } => {
            let treaty = session.make_peace(war)?;
            Ok(vec![format!("Paix signée : {treaty}")])
        }
        Command::Treaty {
            kind,
            first,
            second,
        } => {
            let treaty =
                session.sign_treaty(*kind, &[first.clone(), second.clone()], Some(1080))?;
            Ok(vec![format!("Traité signé : {treaty}")])
        }
        Command::Missile {
            nation,
            missile_type,
            province,
        } => {
            let missile = session.launch_missile(nation, missile_type, province)?;
            Ok(vec![format!("Missile lancé : {missile}")])
        }
        Command::Research { nation, technology } => {
            session.set_research(nation, technology)?;
            Ok(vec![format!(
                "Recherche de {technology} lancée pour {nation}"
            )])
        }
        Command::Event { choice } => {
            let nation = session
                .player_nation()
                .cloned()
                .ok_or_else(|| CoreError::data("aucune nation du joueur"))?;
            session.resolve_event(&nation, choice)?;
            Ok(vec![format!("Choix d'événement appliqué : {choice}")])
        }
        Command::Revolution { nation } => {
            session.force_revolution(nation)?;
            Ok(vec![format!("Révolution forcée pour {nation}")])
        }
        Command::Help => Ok(Command::help_lines()),
    }?;
    for line in &lines {
        host.info(line);
    }
    Ok(lines)
}

/// Analyse puis exécute une ligne de commande.
pub fn execute_str(
    session: &mut Session,
    host: &mut dyn Host,
    input: &str,
) -> CoreResult<Vec<String>> {
    let command = Command::parse(input).map_err(|error| CoreError::data(error.to_string()))?;
    execute(session, host, &command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declarations_natives_utilisent_name_kind() {
        for usage in native_command_usages() {
            assert!(usage.starts_with("civ "));
            for word in usage.split_whitespace().skip(1) {
                if word.starts_with('<') {
                    let inner = word.trim_matches(|character| character == '<' || character == '>');
                    assert!(inner.contains(':'), "argument non typé : {word}");
                } else if word.starts_with('[') {
                    let inner = word.trim_matches(|character| character == '[' || character == ']');
                    assert!(inner.contains(':'), "argument optionnel non typé : {word}");
                }
            }
        }
        assert!(!native_command_usages()
            .iter()
            .any(|usage| usage.contains("<ticks>")
                || usage.contains("<nation>")
                || usage.contains("<province>")
                || usage.contains("<guerre>")
                || usage.contains("<choix>")
                || usage.contains("[nom]")));
    }

    #[test]
    fn analyse_les_commandes() {
        assert_eq!(Command::parse("/civ ping").unwrap(), Command::Ping);
        assert_eq!(Command::parse("civ status").unwrap(), Command::Status);
        assert_eq!(
            Command::parse("/civ tick 42").unwrap(),
            Command::Tick { ticks: 42 }
        );
        assert_eq!(
            Command::parse("/civ save ma partie").unwrap(),
            Command::Save {
                name: "ma partie".to_owned()
            }
        );
        assert!(Command::parse("/civ tick -1").is_err());
        assert!(Command::parse("/civ tick 1 2").is_err());
        assert!(Command::parse("/civ ping extra").is_err());
        assert!(Command::parse("/civ inconnue").is_err());
    }
}
