//! Lanceur exécutable pour Horizon et le mod Civilisations.
//!
//! Ce programme assure le packaging, le déploiement du mod dans l'instance Horizon
//! active et le lancement du jeu avec accélération matérielle et support complet
//! de la simulation géopolitique.

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const HORIZON_INSTANCE_NAME: &str = "civilisation-ia";
const SERVER_MODE_DESCRIPTION: &str = "Simulateur console local (non serveur Horizon)";

fn main() {
    let args: Vec<String> = env::args().collect();

    // Affichage de l'aide
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    // Mode simulation interactive en ligne de commande
    if args
        .iter()
        .any(|a| a == "--sim" || a == "--cli" || a == "-s")
    {
        run_interactive_sim();
        return;
    }

    // Alias historique du simulateur console local.
    if args.iter().any(|a| a == "--server") {
        run_console_simulator();
        return;
    }

    // Mode déploiement seul
    if args.iter().any(|a| a == "--deploy" || a == "-d") {
        print_banner();
        if let Err(e) = deploy_mod() {
            eprintln!("ERREUR lors du déploiement : {}", e);
            std::process::exit(1);
        }
        println!("\n[OK] Déploiement terminé avec succès.");
        return;
    }

    // Mode par défaut : déploiement puis lancement graphique de Horizon
    print_banner();
    println!("[1/2] Préparation et déploiement du mod Civilisations...");
    if let Err(e) = deploy_mod() {
        eprintln!("[AVERTISSEMENT] Déploiement automatique : {}", e);
    } else {
        println!("      Mod natif et pack de données synchronisés avec succès.");
    }

    println!("[2/2] Lancement de Horizon 26.3...");
    let extra_args: Vec<String> = args
        .into_iter()
        .skip(1)
        .filter(|a| !a.starts_with("--version="))
        .collect();

    launch_horizon(&extra_args);
}

fn print_banner() {
    println!("====================================================================");
    println!("            HORIZON 26.3 — MOD CIVILISATIONS (v0.1.0)               ");
    println!("           Plateforme de Géopolitique & Stratégie Voxel             ");
    println!("====================================================================\n");
}

fn print_help() {
    print_banner();
    println!("USAGE :");
    println!("  Civilisations.exe [OPTIONS] [ARGUMENTS DU JEU...]\n");
    println!("MODES D'EXÉCUTION :");
    println!("  (aucun argument)     Déploie le mod et lance le client Horizon 26.3");
    println!("  --sim, --cli, -s     Lance le simulateur géopolitique interactif en console");
    println!("  --server             Lance le {SERVER_MODE_DESCRIPTION}; voir docs/06");
    println!("  --deploy, -d         Met à jour et installe le mod sans lancer le jeu");
    println!("  --help, -h           Affiche cette aide\n");
    println!("EXEMPLES :");
    println!("  .\\Civilisations.exe");
    println!("  .\\Civilisations.exe --world \"Partie Geopolitique\" --creative");
    println!("  .\\Civilisations.exe --sim");
}

/// Trouve la racine du projet Civilisations.
fn find_project_root() -> PathBuf {
    let current_exe = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = current_exe.parent().unwrap_or_else(|| Path::new("."));

    for _ in 0..5 {
        if dir.join("native").join("civ").exists() || dir.join("data").join("civ").exists() {
            return dir.to_path_buf();
        }
        if let Some(parent) = dir.parent() {
            dir = parent;
        } else {
            break;
        }
    }
    // Repli sur le répertoire de travail
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Déploie le mod natif (.hmod / .hnmod) et les données dans l'instance Horizon.
fn deploy_mod() -> io::Result<()> {
    let root = find_project_root();
    let hmod_path = root.join("native").join("civ.hmod");
    let hnmod_path = root.join("native").join("civ.hnmod");

    // S'assurer qu'au moins un bundle (civ.hmod ou civ.hnmod) existe
    if !hmod_path.exists() && !hnmod_path.exists() {
        println!("      Génération du bundle natif via tools/package_native.py...");
        let script = root.join("tools").join("package_native.py");
        if script.exists() {
            let status = Command::new("python").arg(&script).status()?;
            if !status.success() {
                eprintln!("      Attention : la génération via python a échoué.");
            }
        }
    }

    // Répertoire AppData de l'utilisateur
    let appdata = match env::var("APPDATA") {
        Ok(val) => PathBuf::from(val),
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "APPDATA introuvable",
            ))
        }
    };

    let horizon_dir = appdata.join("horizon");
    let instance_dir = horizon_dir.join("instances").join(HORIZON_INSTANCE_NAME);

    // Dossiers cibles pour les mods natifs
    let target_dirs = [
        instance_dir.join("horizon-native-mods"),
        horizon_dir.join("horizon-native-mods"),
    ];

    for target_dir in &target_dirs {
        fs::create_dir_all(target_dir)?;
        if hmod_path.exists() {
            let dest = target_dir.join("civ.hmod");
            fs::copy(&hmod_path, &dest)?;
            println!("      -> Installé dans : {}", dest.display());
        }
        if hnmod_path.exists() {
            let dest = target_dir.join("civ.hnmod");
            fs::copy(&hnmod_path, &dest)?;
            println!("      -> Installé dans : {}", dest.display());
        }
    }

    // Activer mod.civ:true dans horizon-mods.txt
    let instance_mod_list = instance_dir.join("horizon-mods.txt");
    if !instance_mod_list.exists() {
        fs::write(&instance_mod_list, "mod.civ:true\n")?;
    }

    let mod_list_files = [instance_mod_list, horizon_dir.join("horizon-mods.txt")];

    for mod_file in &mod_list_files {
        if mod_file.exists() {
            let content = fs::read_to_string(mod_file).unwrap_or_default();
            if !content.contains("mod.civ:true") {
                let mut updated = content.clone();
                if !updated.ends_with('\n') && !updated.is_empty() {
                    updated.push('\n');
                }
                updated.push_str("mod.civ:true\n");
                let _ = fs::write(mod_file, updated);
                println!("      -> Mod activé dans : {}", mod_file.display());
            }
        }
    }

    // Copier le pack de données sous datapacks
    let source_data = root.join("data").join("civ");
    if source_data.exists() {
        let dest_datapack = instance_dir.join("datapacks").join("civ");
        copy_dir_recursive(&source_data, &dest_datapack)?;
        println!(
            "      -> Données du scénario synchronisées dans : {}",
            dest_datapack.display()
        );
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Lance le jeu Horizon avec les options appropriées.
fn launch_horizon(extra_args: &[String]) {
    let localappdata = env::var("LOCALAPPDATA").unwrap_or_default();
    let appdata = env::var("APPDATA").unwrap_or_default();

    // 1. Chercher le lanceur officiel Horizon Launcher
    let launcher_candidates = [
        PathBuf::from(&localappdata)
            .join("Programs")
            .join("Horizon Launcher")
            .join("horizon-launcher.exe"),
        PathBuf::from(
            r"C:\Users\imano\AppData\Local\Programs\Horizon Launcher\horizon-launcher.exe",
        ),
    ];

    let mut found_launcher = None;
    for cand in &launcher_candidates {
        if cand.exists() {
            found_launcher = Some(cand.clone());
            break;
        }
    }

    // 2. Chercher le binaire client Horizon direct
    let client_candidates = [
        PathBuf::from(&appdata)
            .join("horizon-launcher")
            .join("versions")
            .join("native-fdc87ae-32ba323aa7f1")
            .join("horizon-client.exe"),
        PathBuf::from(&appdata)
            .join("horizon-launcher")
            .join("versions")
            .join("native-fdc87ae-316353329599")
            .join("horizon-client.exe"),
        PathBuf::from(&appdata)
            .join("horizon-launcher")
            .join("versions")
            .join("nightly-2026-11-13-9f42064")
            .join("horizon-client.exe"),
    ];

    let mut found_client = None;
    for cand in &client_candidates {
        if cand.exists() {
            found_client = Some(cand.clone());
            break;
        }
    }

    if let Some(launcher) = found_launcher {
        println!("      Exécutable détecté : {}", launcher.display());
        println!(
            "      Démarrage de l'instance '{}'...\n",
            HORIZON_INSTANCE_NAME
        );

        let mut cmd = Command::new(&launcher);
        cmd.arg("play").arg("--instance").arg(HORIZON_INSTANCE_NAME);

        for arg in extra_args {
            cmd.arg(arg);
        }

        match cmd.status() {
            Ok(status) => {
                println!(
                    "\nSession Horizon terminée avec le code : {:?}",
                    status.code()
                );
            }
            Err(e) => {
                eprintln!("ERREUR lors de l'exécution du lanceur : {}", e);
            }
        }
    } else if let Some(client) = found_client {
        println!(
            "      Lanceur non trouvé, démarrage direct via : {}",
            client.display()
        );
        let client_dir = client.parent().unwrap_or_else(|| Path::new("."));

        let mut cmd = Command::new(&client);
        cmd.current_dir(client_dir);
        for arg in extra_args {
            cmd.arg(arg);
        }

        match cmd.status() {
            Ok(status) => {
                println!(
                    "\nSession Horizon terminée avec le code : {:?}",
                    status.code()
                );
            }
            Err(e) => {
                eprintln!("ERREUR lors de l'exécution du client : {}", e);
            }
        }
    } else {
        eprintln!("\nERREUR : Horizon n'a pas été trouvé sur ce système.");
        eprintln!(
            "Installez Horizon depuis https://playhorizon.dev ou lancez le simulateur avec --sim."
        );
    }
}

/// Exécute le simulateur interactif en console.
fn run_interactive_sim() {
    print_banner();
    println!("--- MODE SIMULATEUR GÉOPOLITIQUE INTERACTIF ---");
    println!("Chargement du pack de données officiel 'Arc des Frontières'...\n");

    let root = find_project_root();
    let data_dir = root.join("data").join("civ");

    if !data_dir.exists() {
        eprintln!(
            "ERREUR : répertoire data/civ introuvable : {}",
            data_dir.display()
        );
        return;
    }

    match civ_horizon::data::loader::load_data_pack(&data_dir) {
        Ok((_manifest, _registries, scenario, game_data)) => {
            let session_res = civ_horizon::state::Session::new_with_data(
                &scenario,
                game_data,
                "e925cf06-28b5-45c3-a8e6-123456789abc",
                "Partie Interactive",
                "2026-01-01T00:00:00Z",
                20,
            );

            let mut session = match session_res {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ERREUR lors de l'initialisation de la session : {:?}", e);
                    return;
                }
            };

            let mut host = civ_horizon::host::RecordingHost::default();

            println!("[OK] Scénario chargé : {}", scenario.name.resolve());
            println!("     Nations initiales : {}", scenario.nations.len());
            println!(
                "     Provinces         : {}",
                scenario.world.provinces.len()
            );
            println!("     Date initiale     : {}\n", session.status().date);
            println!("Tapez une commande (/civ help, /civ nations, /civ nation <id>, /civ tick <n>, quit) :");

            let stdin = io::stdin();
            let mut input = stdin.lock();
            let mut stdout = io::stdout();

            loop {
                let line = match read_console_line(&mut input, &mut stdout, "civ> ") {
                    Ok(Some(line)) => line,
                    Ok(None) => {
                        println!("Fin de l'entrée console; fermeture du simulateur.");
                        break;
                    }
                    Err(error) => {
                        eprintln!("Lecture console impossible : {error}");
                        break;
                    }
                };
                if line.trim().is_empty() {
                    continue;
                }
                let input = line.trim();

                if input == "quit" || input == "exit" || input == "/civ quit" {
                    println!("Fermeture du simulateur.");
                    break;
                }

                match civ_horizon::commands::Command::parse(input) {
                    Ok(cmd) => {
                        match civ_horizon::commands::execute(&mut session, &mut host, &cmd) {
                            Ok(lines) => {
                                for line in lines {
                                    println!("{}", line);
                                }
                            }
                            Err(e) => eprintln!("Erreur d'exécution : {}", e),
                        }
                    }
                    Err(e) => eprintln!("Erreur de syntaxe : {}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("ERREUR lors du chargement des données : {:?}", e);
        }
    }
}

fn read_console_line<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    prompt: &str,
) -> io::Result<Option<String>> {
    write!(output, "{prompt}")?;
    output.flush()?;

    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line))
}

/// Exécute le simulateur console local sélectionné par l'alias historique --server.
fn run_console_simulator() {
    print_banner();
    println!("--- MODE {SERVER_MODE_DESCRIPTION} ---");
    println!(
        "Aucune écoute réseau, aucun natif Horizon, aucune sauvegarde serveur n'est démarrée."
    );
    println!("Initialisation du moteur de simulation (voir docs/06-dedicated-server.md)...");

    let root = find_project_root();
    let data_dir = root.join("data").join("civ");

    match civ_horizon::data::loader::load_data_pack(&data_dir) {
        Ok((_manifest, _registries, scenario, game_data)) => {
            let session_res = civ_horizon::state::Session::new_with_data(
                &scenario,
                game_data,
                "e925cf06-28b5-45c3-a8e6-123456789abc",
                "Simulateur Console",
                "2026-01-01T00:00:00Z",
                20,
            );

            let mut session = match session_res {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ERREUR : {:?}", e);
                    return;
                }
            };

            let mut host = civ_horizon::host::RecordingHost::default();

            println!(
                "[OK] Simulateur prêt. Date fictive courante : {}",
                session.status().date
            );
            println!(
                "Simulateur en attente de commandes console (tapez '/civ quit' pour quitter) :"
            );

            let stdin = io::stdin();
            let mut input = stdin.lock();
            let mut stdout = io::stdout();

            loop {
                let line = match read_console_line(&mut input, &mut stdout, "sim:civ> ") {
                    Ok(Some(line)) => line,
                    Ok(None) => {
                        println!("Fin de l'entrée console; arrêt du simulateur.");
                        break;
                    }
                    Err(error) => {
                        eprintln!("Lecture console impossible : {error}");
                        break;
                    }
                };
                if line.trim().is_empty() {
                    continue;
                }
                let input = line.trim();

                if input == "quit" || input == "stop" || input == "/civ quit" {
                    println!("Arrêt du simulateur console.");
                    break;
                }

                match civ_horizon::commands::Command::parse(input) {
                    Ok(cmd) => {
                        match civ_horizon::commands::execute(&mut session, &mut host, &cmd) {
                            Ok(lines) => {
                                for line in lines {
                                    println!("[Simulateur] {}", line);
                                }
                            }
                            Err(e) => eprintln!("[Simulateur] Erreur d'exécution : {}", e),
                        }
                    }
                    Err(e) => eprintln!("[Simulateur] Erreur de syntaxe : {}", e),
                }
            }
        }

        Err(e) => {
            eprintln!("ERREUR : {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_input_returns_eof_instead_of_spinning() {
        let mut input = io::Cursor::new(Vec::<u8>::new());
        let mut output = Vec::new();

        assert_eq!(
            read_console_line(&mut input, &mut output, "civ> ").unwrap(),
            None
        );
        assert_eq!(output, b"civ> ");
    }

    #[test]
    fn console_input_returns_lines_and_writes_prompt() {
        let mut input = io::Cursor::new(b"/civ status\n".to_vec());
        let mut output = Vec::new();

        assert_eq!(
            read_console_line(&mut input, &mut output, "sim:civ> ").unwrap(),
            Some("/civ status\n".to_owned())
        );
        assert_eq!(output, b"sim:civ> ");
    }

    #[test]
    fn server_alias_is_explicitly_not_horizon_server() {
        assert!(SERVER_MODE_DESCRIPTION.contains("non serveur Horizon"));
        assert!(SERVER_MODE_DESCRIPTION.contains("console"));
    }
}
