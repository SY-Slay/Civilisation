# 07 — Feuille de route

## Règle de progression

Chaque étape est livrée complète, testée et documentée. **Aucune étape suivante n'est commencée sans un accord explicite.** L'étape 0 a été validée par le bundle `/civ ping`. L'étape 1 a livré l'arbre, les schémas et les fixtures. L'étape 2 a livré le socle Rust et ses tests. L'étape 3 a livré la boucle géopolitique déterministe et les commandes associées.

## Étape 0 — Preuve native `/civ ping` — terminée

- Ancien bundle `.hnmod` natif épinglé sur Horizon `89f236e...`.
- `horizon-native.json`, `horizon-mixin.toml`, module runtime, hooks de commande.
- `/civ ping` répond `Civ : pong`.
- Build et installation documentés.
- Limite : pas encore exécuté dans une instance Horizon installée ; le launcher et le build restent à faire côté utilisateur.

## Étape 1 — Arbre + schémas — terminée

- Arborescence complète du projet.
- 24 schémas JSON Draft 2020-12.
- 22 fixtures valides, validées avec `jsonschema 4.26.0`.
- Documentation : politique de contenu, architecture, modèle de données, sauvegarde, APIs vérifiées.
- Aucun code Rust, aucun pseudo-code.

## Étape 2 — Socle `civ-core` + `civ-horizon` — terminée

Objectif : une simulation testable sans Horizon.

- Workspace Cargo local avec `civ-core` et `civ-horizon`.
- `civ-core` : IDs typés, temps fictif, RNG SplitMix64 à compteur, géométrie, monde, nations, économie, armée, technologie, diplomatie, guerre, révolutions, missiles, événements, validation métier, sauvegarde.
- Scénario minimal chargé depuis les fixtures et validé par les règles métier.
- Sauvegarde snapshot complète, checksum SHA-256 canonique, migration v1, copie `.bak` et chargement vérifié.
- `civ-horizon` : trait `Host`, session, commandes `/civ`, sauvegarde, `ExplosionEffects`, modèles d'UI, protocole réseau.
- Aucun hook Horizon dans `civ-core`; aucun type Horizon dans `civ-horizon`.
- Critère de sortie atteint : 44 tests verts à l'époque, deux sessions identiques produisent le même checksum, une sauvegarde se recharge à l'identique.
- Limite : l'IA, les combats, les révolutions et les missiles n'étaient pas encore simulés tick par tick ; l'étape 2 fournissait leurs structures, invariants et points d'entrée.
- Limite : `glue.rs`, les hooks natifs et les écrans restent à implémenter à l'étape 4.

## Étape 3 — Boucle géopolitique — terminée

- Économie, population, stabilité, technologie : `sim/economy.rs`.
- Diplomatie active : relations, traités, sanctions, déclaration de guerre et paix.
- IA utility déterministe : budget militaire, recherche, sanctions, traités et guerre.
- Guerre : fronts, batailles, occupation, usure et paix automatique.
- Révolutions : pression, progression, factions, guerre civile et issues.
- Missiles : lancement, portée, trajectoire, interception et rapports de dégâts.
- Événements : triggers, choix, cooldowns, effets immédiats et différés persistants.
- Commandes `/civ` : nations, fiche, diplomatie, guerre, paix, traité, missile, recherche, événement, révolution.
- Données JSON `GameData` : technologies, missiles et événements validés.
- Sauvegarde : `start_date`, `difficulty`, `ticks_per_day` et `event_state.pending` conservés ; reprise déterministe après rechargement.
- Critère de sortie atteint : partie de 10 ans fictifs reproductible avec guerre, traité, révolution et fin de guerre ; 54 tests verts.
- Limite : l'adaptateur Horizon, les hooks natifs et les écrans restent à implémenter à l'étape 4.
- Limite : Orbital Laser n'expose pas de commande de frappe vérifiée ; seule l'intégration visuelle reste réaliste.

## Étape 4 — Adaptateur Horizon — terminée

- `glue.rs` : implémentation de `Host` connectant Horizon à `civ-horizon` et `civ-core`.
- Injections complètes de 18 hooks : commandes joueur et console, ticks serveur/client, cycle de vie du monde, HUD, carte 3D, écrans de mod, caméra, shake, main et réseau.
- Sauvegarde sous `<world_dir>/civ/<save>.json`, écriture atomique avec copie `.bak`.
- Modèles et widgets d'UI spécialisés : carte interactive, fiches nation, diplomatie, théâtre de guerre, commandement balistique.
- Rendu 3D : frontières polygonales par nation/occupation, marqueurs de capitales et fronts, trajectoires balistiques paraboliques interpolées.
- Système cinématique : directeur avec file d'attente de plans (Stationary, Flyover, Orbit), letterbox cinémascope (2.39:1), sous-titres, clamp de sécurité par rapport à l'ancre du joueur pour éviter les chunks déchargés.
- Protocole réseau : validation des paquets (bornés à 28 Ko), synchronisation autoritaire serveur -> clients.
- Packaging historique : outil reproductible `tools/package_native.py` produisant `native/civ.hmod`.
- Critère de sortie atteint : 67 tests verts, 0 avertissement Clippy, formatage strict respecté, bundle natif fonctionnel.

## Étape 5 — Contenu et scénario — terminée

- Pack de données complet produit sous `data/civ/` avec manifeste `manifest.json` (licence CC-BY-SA-4.0).
- Scénario « Arc des Frontières » : 4 nations fictives (Ardenne, Kesh, Varenne, Orlune), 16 provinces irrégulières, 24 frontières naturelles partagées, 6 relations bilatérales.
- Registres de contenu complets : 4 cultures, 4 religions, 4 idéologies, 4 gouvernements, 5 ressources économiques et stratégiques.
- Arbres technologiques (6 technologies), unités militaires (5 types), missiles (3 types), événements scénarisés (5 événements), 4 réservoirs de noms et localisation FR/EN.
- Outil de validation métier `tools/validate_data.py` vérifiant la conformité JSON Schema, l'intégrité référentielle, l'irrégularité des frontières et le respect absolu de la politique de contenu 100% fictionnelle.
- Test d'intégration Rust `native/civ/mixins/civ/civ-core/tests/datapack.rs` validant le chargement via `load_data_pack` et le bon déroulement de la simulation.
- Assets de production originaux sous `assets/civ/` (icônes, drapeaux PNG 100% originaux, textures, localisation).
- Bundle natif actualisé intégrant les fichiers de code, données et assets; le packaging actuel produit `.hmod`.
- Critère de sortie atteint : 69 tests verts, 0 avertissement Clippy, formatage strict, validation de données et schémas sans faille.

## Étape 6 — Finition et diffusion — terminée

- Tests de non-régression déterministes multi-annuels (`native/civ/mixins/civ/civ-core/tests/golden.rs`) :
  - Simulation de 5 ans fictifs (1800 jours) exécutée deux fois de manière totalement indépendante avec comparaison stricte du journal d'événements et du checksum SHA-256 canonique scellé.
  - Test de reprise après sauvegarde : une simulation arrêtée et sauvegardée à 1 an puis reprise jusqu'à 3 ans produit exactement le même état et checksum qu'une simulation continue ininterrompue de 3 ans.
- Tests d'intégration de migration de sauvegarde (`native/civ/mixins/civ/civ-core/tests/migration.rs`) :
  - Rejet des versions de schéma non supportées (v0, v99).
  - Migration automatique du nom d'algorithme RNG legacy (`splitmix64_pcg_v1` vers `splitmix64_counter_v1`).
  - Validation du re-scellement et de la reprise du RNG depuis l'état migré.
  - Vérification de l'idempotence de la migration sur les sauvegardes déjà à jour.
- Outil hors ligne `tools/migrate_save.py` : inspection, migration et validation Draft 2020-12 des sauvegardes JSON.
- Note d'état sur le serveur dédié (`docs/06-dedicated-server.md`) : les hooks de tick, de cycle de vie, de console et de réseau existent, mais le fonctionnement sur serveur headless n'est pas vérifié. Ne pas traiter cette note comme une procédure d'installation.
- Guide complet de création de packs de données tiers (`docs/08-pack-creation-guide.md`) : arborescence, schémas, règles géométriques des provinces (polygones simples, aire > 1024, réciprocité, pas de segment droit > 160 blocs) et respect absolu de la politique de contenu 100% fictionnelle.
- Formalisation des licences (`LICENSE`) à la racine du dépôt : double licence MPL-2.0 (code logiciel) et CC-BY-SA-4.0 (données de jeu, scénarios, documentation, assets artistiques).
- Packaging final et reproductibilité : `tools/package_native.py` produisant le bundle natif `native/civ.hmod` (empreinte SHA-256 stable).
- Vérifications locales les plus récentes consignées : 80 tests `civ-core`/`civ-horizon`, 3 tests launcher, 0 avertissement Clippy, formatage strict, 24 schémas et 67 fichiers de données valides.

## Risques connus

| Risque | Impact | Réponse |
|---|---|---|
| API Horizon non stable entre commits | Build cassé | Épingler le commit, tester chaque hook, isoler dans `glue.rs` |
| Caméra cinématique et chunks non chargés | Plans vides | Rester près du joueur, charger avant, limiter les plans |
| Taille des sauvegardes | Temps de chargement | Compression, snapshot complet en v1, optimisation ensuite |
| Orbital Laser sans API de frappe | Intégration limitée | Effets visuels seulement, repli particules/sons |
| Performances du rendu de frontières | FPS | Index spatial, niveau de détail, budgets de dessin |
| Contenu réel accidentel | Non-conformité | Politique de contenu, revue, validation par liste de noms |
| Déterminisme cassé par un `HashMap` | Parties non reproductibles | Tri explicite, tests golden, revue de code |

## Statut du projet

Les étapes 0 à 6 sont livrées pour le périmètre local documenté. Les tests locaux et le lancement solo décrits dans `JOURNAL_IA.md` ne valident pas le serveur dédié; le support headless reste à vérifier. Les changements de code postérieurs au dernier build natif devront aussi être reconstruits et vérifiés avant une nouvelle diffusion.
