# Civilisations — arbre complet du projet

**État : étapes 0 à 6 livrées pour le périmètre local vérifié.** L'étape 0 a livré le bundle `/civ ping`. L'étape 1 a livré l'arborescence, 24 schémas et 22 fixtures. L'étape 2 a livré `civ-core` et `civ-horizon`. L'étape 3 a livré la boucle géopolitique déterministe, les données JSON et les commandes `/civ`. L'étape 4 a livré l'adaptateur Horizon (`glue.rs`, 18 injections de hooks, rendu 3D des frontières, directeur cinématique, écrans spécialisés et packaging `.hmod`), vérifié en monde solo avant le correctif de carte actuel. L'étape 5 a livré le pack de données officiel sous `data/civ/` (scénario « Arc des Frontières », 4 nations, 16 provinces, 24 frontières, registres complets, assets sous `assets/civ/`). L'étape 6 a livré les tests golden 5 ans déterministes, la reprise après sauvegarde, les tests de migration Rust et Python (`tools/migrate_save.py`), le guide de création de packs (`docs/08-pack-creation-guide.md`), et la double licence MPL-2.0 / CC-BY-SA-4.0 (`LICENSE`). Le support serveur dédié n'est pas vérifié; voir `docs/06-dedicated-server.md`. 80 tests verts dans `civ-core/civ-horizon`, 3 tests verts dans le launcher, 0 warning Clippy, formatage Rust conforme; le packaging actuel cible `native/civ.hmod`.

Légende :

- `[x]` : présent dans le dépôt et compilé/testé.
- `[ ]` : prévu, pas encore créé.
- Les chemins `native/...` suivent la disposition d'un bundle natif `.hmod` : `horizon-native.json` à la racine, puis `mixins/<id>/`.
- `civ-ping/` est la preuve indépendante de l'étape 0.

## Racine

```text
C:\Users\imano\Downloads\Horizon\
├─ Cargo.toml                                  [x] workspace civ-core + civ-horizon
├─ Cargo.lock                                  [x] verrouillé pour des builds reproductibles
├─ README.md                                   [x] présentation, statut, build, limites
├─ PROJECT_TREE.md                             [x] ce document
├─ .gitignore                                  [x] target/, sauvegardes, temporaires
├─ LICENSE                                     [x] double licence MPL-2.0 et CC-BY-SA-4.0
├─ civ-ping/                                   [x] étape 0 : /civ ping natif
│  ├─ horizon-native.json                      [x]
│  ├─ mixins/civ/horizon-mixin.toml            [x]
│  ├─ mixins/civ/runtime.rs                    [x]
│  ├─ mixins/civ/register_commands.rs          [x]
│  ├─ mixins/civ/perform_command.rs            [x]
│  ├─ package.py                               [x]
│  ├─ README.md                                [x]
│  └─ civ.hmod                                 [x] bundle empaqueté
├─ docs/
│  ├─ 00-content-policy.md                     [x] fiction, licences, frontières, IA
│  ├─ 01-architecture.md                       [x] séparation core / adapter / glue
│  ├─ 02-data-model.md                         [x] modèle de données et validation
│  ├─ 03-save-format.md                        [x] sauvegarde, checksum, migration
│  ├─ 04-commands.md                           [x] /civ, syntaxe, autorité
│  ├─ 05-horizon-api.md                        [x] APIs vérifiées et limites
│  ├─ 06-dedicated-server.md                   [x] état et limites du support serveur dédié
│  ├─ 07-roadmap.md                            [x] étapes locales et limites du serveur dédié
│  └─ 08-pack-creation-guide.md                [x] guide de création et validation de packs tiers
├─ schemas/                                    [x] 24 schémas Draft 2020-12
│  ├─ common.schema.json                       [x]
│  ├─ index.schema.json                        [x]
│  ├─ data-pack.schema.json                    [x]
│  ├─ config.schema.json                       [x]
│  ├─ world.schema.json                        [x]
│  ├─ province.schema.json                     [x]
│  ├─ culture.schema.json                      [x]
│  ├─ religion.schema.json                     [x]
│  ├─ ideology.schema.json                     [x]
│  ├─ government.schema.json                   [x]
│  ├─ resource.schema.json                     [x]
│  ├─ nation.schema.json                       [x]
│  ├─ diplomacy.schema.json                    [x]
│  ├─ war.schema.json                          [x]
│  ├─ revolution.schema.json                   [x]
│  ├─ missile.schema.json                      [x]
│  ├─ technology.schema.json                   [x]
│  ├─ unit.schema.json                         [x]
│  ├─ event.schema.json                        [x]
│  ├─ game-data.schema.json                    [x] agrégat technologies/missiles/événements
│  ├─ names.schema.json                        [x]
│  ├─ localization.schema.json                 [x]
│  ├─ scenario.schema.json                     [x]
│  └─ save.schema.json                         [x]
├─ tests/
│  ├─ schemas/
│  │  ├─ README.md                             [x] fixtures et commandes de validation
│  │  ├─ valid/                                [x] 22 fixtures valides
│  │  │  ├─ config.json                        [x]
│  │  │  ├─ culture.json                       [x]
│  │  │  ├─ data-pack.json                     [x]
│  │  │  ├─ diplomacy.json                     [x]
│  │  │  ├─ event.json                         [x]
│  │  │  ├─ game-data.json                     [x]
│  │  │  ├─ government.json                    [x]
│  │  │  ├─ ideology.json                      [x]
│  │  │  ├─ index.json                         [x]
│  │  │  ├─ localization.json                  [x]
│  │  │  ├─ missile.json                       [x]
│  │  │  ├─ names.json                         [x]
│  │  │  ├─ nation.json                        [x]
│  │  │  ├─ religion.json                      [x]
│  │  │  ├─ resource.json                      [x]
│  │  │  ├─ revolution.json                    [x]
│  │  │  ├─ scenario.json                      [x]
│  │  │  ├─ save.json                          [x]
│  │  │  ├─ technology.json                    [x]
│  │  │  ├─ unit.json                          [x]
│  │  │  ├─ war.json                           [x]
│  │  │  └─ world.json                         [x]
│  │  └─ invalid/                              [ ] cas de rejet JSON Schema volontaires
│  ├─ core/                                    [ ] tests d'intégration dédiés à civ-core
│  ├─ golden/                                  [ ] parties de référence déterministes
│  └─ integration/                             [ ] Horizon, serveur dédié, sauvegardes
├─ tools/
│  ├─ validate_schemas.py                      [x] validation des schémas/fixtures/sauvegardes
│  ├─ validate_data.py                         [x] validation métier des données (schémas, géométrie, fiction)
│  ├─ package_native.py                        [x] packaging .hmod reproductible
│  ├─ build_data_pack.py                       [x] générateur déterministe du pack et des assets
│  └─ migrate_save.py                          [x] migration et validation hors ligne des sauvegardes
└─ data/
   └─ civ/                                     [x] pack de données de production officiel
```

## Bundle natif

```text
native/civ/
├─ horizon-native.json                         [x] manifeste natif (étape 4)
└─ mixins/civ/
   ├─ horizon-mixin.toml                       [x] crates + modules + injections + hooks
   ├─ glue.rs                                  [x] seule couche Horizon (étape 4)
   ├─ register_commands.rs                     [x] native_register_commands
   ├─ perform_command.rs                       [x] native_perform_command
   ├─ console_command.rs                       [x] native_perform_console_command
   ├─ server_tick.rs                           [x] native_server_tick
   ├─ client_tick.rs                           [x] native_client_tick_pre/post
   ├─ world_lifecycle.rs                       [x] native_world_lifecycle
   ├─ world_drawn.rs                           [x] native_world_drawn
   ├─ hud.rs                                   [x] native_hud
   ├─ screen_open.rs                           [x] native_mod_screen_open
   ├─ screen_render_post.rs                    [x] native_screen_render_post
   ├─ screen_click.rs                          [x] native_screen_mouse_button_pressed_pre
   ├─ screen_key.rs                            [x] native_screen_key_pressed_pre
   ├─ screen_char.rs                           [x] native_screen_char_typed_pre
   ├─ camera_pose.rs                           [x] native_camera_pose
   ├─ camera_shake.rs                          [x] native_camera_shake
   ├─ render_hand.rs                           [x] native_render_hand
   ├─ net_from_client.rs                       [x] native_net_from_client
   ├─ net_from_server.rs                       [x] native_net_from_server
   ├─ civ-core/                                [x] simulation pure, sans Horizon
   │  ├─ Cargo.toml                            [x] serde, serde_json, sha2, thiserror
   │  ├─ src/
   │  │  ├─ lib.rs                             [x] API publique et modules
   │  │  ├─ error.rs                           [x] erreurs typées
   │  │  ├─ ids.rs                             [x] identifiants typés validés
   │  │  ├─ time.rs                            [x] date fictive, Tick, SimClock
   │  │  ├─ rng.rs                             [x] SplitMix64 à compteur
   │  │  ├─ math.rs                            [x] Vec2/Vec3, polygones, centroïdes
   │  │  ├─ text.rs                            [x] texte localisé avec repli
   │  │  ├─ effects.rs                         [x] Condition, Effect, ScheduledEffect
   │  │  ├─ data.rs                            [x] GameData JSON : technologies, missiles, événements
   │  │  ├─ scenario.rs                        [x] scénario, difficulté, chargement
   │  │  ├─ event.rs                           [x] définitions et état des événements
   │  │  ├─ validation.rs                      [x] invariants monde/nations/diplomatie/guerres
   │  │  ├─ world/
   │  │  │  ├─ mod.rs                          [x] World et réexports
   │  │  │  ├─ map.rs                          [x] index spatial et validation
   │  │  │  ├─ province.rs                     [x] propriété, contrôle, population
   │  │  │  ├─ polygon.rs                      [x] aire, simplicité, centroïde
   │  │  │  ├─ border.rs                       [x] frontières partagées
   │  │  │  ├─ terrain.rs                      [x] terrain, climat, coûts
   │  │  │  └─ resources.rs                    [x] gisements et extraction
   │  │  ├─ nation/
   │  │  │  ├─ mod.rs                          [x] NationState
   │  │  │  ├─ model.rs                        [x] gouvernement, dirigeant, revendications
   │  │  │  ├─ economy.rs                      [x] PIB, trésor, secteurs
   │  │  │  ├─ military.rs                     [x] manpower, doctrines, stocks
   │  │  │  └─ technology.rs                   [x] recherche et déblocages
   │  │  ├─ diplomacy/
   │  │  │  ├─ mod.rs                          [x] DiplomacyState
   │  │  │  ├─ relation.rs                     [x] opinion, confiance, peur
   │  │  │  ├─ treaty.rs                       [x] traités et violations
   │  │  │  ├─ organization.rs                 [x] alliances, blocs, votes
   │  │  │  ├─ espionage.rs                    [x] opérations et exposition
   │  │  │  ├─ sanction.rs                     [x] sanctions
   │  │  │  ├─ guarantee.rs                    [x] garanties
   │  │  │  └─ trade.rs                        [x] routes commerciales
   │  │  ├─ war/mod.rs                         [x] guerres, fronts, batailles, armées, paix
   │  │  ├─ revolution/mod.rs                  [x] factions, soutien, guerre civile
   │  │  ├─ missile/mod.rs                     [x] types, instances, trajectoires, dégâts
   │  │  ├─ sim/
   │  │  │  ├─ mod.rs                          [x] boucle quotidienne, actions, reprise de sauvegarde
   │  │  │  ├─ economy.rs                      [x] économie, population, stabilité, recherche
   │  │  │  ├─ ai.rs                           [x] IA utility déterministe
   │  │  │  ├─ diplomacy.rs                    [x] traités, sanctions, guerres, paix
   │  │  │  ├─ warfare.rs                      [x] fronts, batailles, occupation, paix automatique
   │  │  │  ├─ revolution.rs                   [x] pression, progression, guerre civile, issues
   │  │  │  ├─ missile.rs                      [x] lancement, vol, interception, dégâts
   │  │  │  └─ events.rs                       [x] triggers, choix, effets différés
   │  │  └─ save/
   │  │     ├─ mod.rs                          [x] Save, compatibilité, écriture atomique
   │  │     ├─ checksum.rs                     [x] SHA-256 canonique
   │  │     └─ migrate.rs                      [x] version 1 et renommage RNG
   │  └─ tests/
   │     ├─ fixtures/scenario.json             [x] scénario minimal
   │     ├─ fixtures/game_data.json            [x] technologies, missiles, événements de test
   │     ├─ geopolitics.rs                     [x] 4 tests : 10 ans, déterminisme, rechargement, données
   │     ├─ datapack.rs                        [x] 2 tests : chargement pack officiel et simulation
   │     ├─ golden.rs                          [x] 2 tests : golden 5 ans et reprise déterminisme
   │     └─ migration.rs                       [x] 5 tests : rejet v0, migration RNG, idempotence
   └─ civ-horizon/                             [x] adaptateur testable, sans Horizon
      ├─ Cargo.toml                            [x] dépend uniquement de civ-core
      ├─ src/
      │  ├─ lib.rs                             [x] API de l'adaptateur
      │  ├─ host.rs                            [x] trait Host + RecordingHost
      │  ├─ state.rs                           [x] Session, horloge, sauvegarde
      │  ├─ commands.rs                        [x] parsing et exécution /civ
      │  ├─ save.rs                            [x] SaveFile
      │  ├─ effects.rs                         [x] ExplosionEffects + défaut
      │  ├─ ui/
      │  │  ├─ mod.rs                          [x] modèles carte/nations/guerres/missiles
      │  │  ├─ theme.rs                        [x] palette et typographie
      │  │  ├─ widgets.rs                      [x] primitives et layout
      │  │  ├─ map.rs                          [x] vue carte interactive
      │  │  ├─ nation.rs                       [x] vue détaillée de nation
      │  │  ├─ diplomacy.rs                    [x] vue diplomatique
      │  │  ├─ war.rs                          [x] vue des opérations militaires
      │  │  └─ missile.rs                      [x] vue de commandement stratégique
      │  ├─ render/
      │  │  ├─ mod.rs                          [x] primitives 3D et lots de rendu
      │  │  ├─ borders.rs                      [x] tracé 3D des frontières
      │  │  ├─ markers.rs                      [x] marqueurs de monde
      │  │  └─ missiles.rs                     [x] trajectoires balistiques
      │  ├─ cinematic/
      │  │  ├─ mod.rs                          [x] poses de caméra
      │  │  ├─ shots.rs                        [x] plans et mouvements
      │  │  ├─ overlay.rs                      [x] letterbox et sous-titres
      │  │  └─ director.rs                     [x] directeur cinématique
      │  └─ net/
      │     ├─ mod.rs                          [x] messages JSON bornés
      │     ├─ protocol.rs                     [x] intentions et limites réseau
      │     └─ sync.rs                         [x] synchronisation serveur/clients
      └─ tests/
         ├─ session.rs                         [x] 6 tests d'intégration
         └─ actions.rs                         [x] 2 tests commandes/actions Step 3
```

## Livrables des étapes 5 et 6

Tous les fichiers des étapes 5 et 6 sont créés, validés et testés :

### Étape 5 — contenu et data-pack complet

```text
civ-core/src/data/
├─ mod.rs                                    [x] déclarations et GameData
├─ loader.rs                                 [x] chargeur complet load_data_pack
├─ registry.rs                               [x] registres typés cultures/religions/etc.
└─ validation.rs                             [x] validation croisée des registres

civ-core/tests/
└─ datapack.rs                               [x] test d'intégration du pack officiel

tools/
├─ validate_data.py                          [x] validation métier des données (schémas, géométrie, fiction)
└─ build_data_pack.py                        [x] générateur du pack de production
```

### Étape 6 — finition et diffusion

```text
tools/migrate_save.py                        [x] migration hors ligne et conformité save.schema.json
civ-core/tests/golden.rs                     [x] golden test 5 ans et reprise déterminisme
civ-core/tests/migration.rs                  [x] tests d'intégration de migration et idempotence
docs/06-dedicated-server.md                  [x] état documenté; support dédié non vérifié
docs/08-pack-creation-guide.md               [x] guide de création et validation de packs tiers
LICENSE                                      [x] licences MPL-2.0 et CC-BY-SA-4.0
```

## Données éditables de production

```text
data/civ/
├─ manifest.json                             [x] manifeste data-pack officiel (CC-BY-SA-4.0)
├─ config.toml                               [x] configuration par défaut
├─ scenarios/arc_des_frontieres.json         [x] scénario « Arc des Frontières » (16 provinces, 4 nations)
├─ nations/                                  [x] 4 nations fictives (ardenne, kesh, varenne, orlune)
├─ provinces/                                [x] 16 provinces polygonales irrégulières
├─ cultures/                                 [x] 4 cultures fictives (ardennic, keshi, varennois, orlunian)
├─ religions/                                [x] 4 religions fictives (rationalism, martial_order, pragmatism, mysticism)
├─ ideologies/                               [x] 4 idéologies fictives (civic_republicanism, militarist_autocracy, commercial_oligarchy, devotional_theocracy)
├─ governments/                              [x] 4 gouvernements (republic, autocracy, oligarchy, theocracy)
├─ resources/                                [x] 5 ressources (iron_ore, coal, grain, oil, rare_earths)
├─ technologies/                             [x] 6 technologies réparties sur 4 ères
├─ units/                                    [x] 5 unités militaires
├─ missiles/                                 [x] 3 types balistiques / croisière
├─ events/                                   [x] 5 événements narratifs et géopolitiques
├─ names/                                    [x] 4 réservoirs de noms fictifs
└─ localization/                             [x] fr_fr.json, en_us.json
```

## Assets

```text
assets/civ/
├─ icon.png                                  [x] icône du mod (PNG original 64x64)
├─ README.md                                 [x] licences CC-BY-SA-4.0 et conformité
├─ textures/flags/                           [x] 4 drapeaux PNG originaux (ardenne, kesh, varenne, orlune)
├─ textures/map/, ui/                        [x] dossiers de textures
├─ models/                                   [x] modèles
├─ sounds/                                   [x] effets sonores
├─ particles/                                [x] définitions de particules
└─ lang/                                     [x] fr_fr.json, en_us.json
```

## Règles de l'arbre

1. `civ-core` ne dépend ni de Horizon, ni de Minecraft, ni d'un moteur de rendu.
2. `civ-horizon` ne dépend pas de `horizon-client` ; il définit le trait `Host`.
3. `glue.rs` et les fichiers d'injection seront la seule couche qui connaît les APIs Horizon.
4. Les données de référence sont du JSON/TOML, validées par les schémas avant chargement.
5. Les sauvegardes sont des fichiers JSON sous le répertoire du monde, avec checksum, copie `.bak` et migration.
6. Les frontières de production sont irrégulières et naturelles ; la validation métier le vérifie.
7. Aucune nation, aucun dirigeant, aucun territoire réel n'est représenté.
8. Une étape suivante ne commence qu'après autorisation explicite.
