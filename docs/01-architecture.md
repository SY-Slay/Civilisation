# 01 — Architecture

## Objectif

Séparer une **simulation géopolitique pure** d'un **adaptateur Horizon mince**. La simulation doit pouvoir être testée sans lancer Minecraft, et la couche Horizon ne doit contenir que la traduction entre les données du mod et les APIs réelles du jeu.

## Trois couches

Architecture : la boucle de simulation complète est implémentée dans `civ-core` et exposée par `civ-horizon`, connectée à Horizon via `glue.rs` et les mixins natifs.

```text
┌──────────────────────────────────────────────────────────────────┐
│  glue.rs + fichiers d'injection (native/civ/mixins/civ)          │
│  Seule couche qui connaît horizon-client, les hooks et le réseau.│
│  Traduit les events Horizon -> commandes civ-core.               │
│  Traduit les snapshots civ-core -> HUD, écrans, caméra, sons.    │
└───────────────────────────────┬──────────────────────────────────┘
                                │ trait Host + messages
┌───────────────────────────────▼──────────────────────────────────┐
│  civ-horizon (crate Rust pure)                                   │
│  Adaptateur : commandes, session, sauvegarde, UI models,         │
│  trait ExplosionEffects ; rendu et cinématiques à l'étape 4.     │
│  Aucune dépendance à horizon-client ni à Minecraft.              │
└───────────────────────────────┬──────────────────────────────────┘
                                │ types publics, commandes, snapshots
┌───────────────────────────────▼──────────────────────────────────┐
│  civ-core (crate Rust pure)                                      │
│  Monde, nations, diplomatie, guerre, révolutions, missiles,      │
│  events, économie, sauvegarde ; IA active déterministe.          │
│  Aucune dépendance Horizon.                                      │
│  Déterministe, testable, sérialisable.                           │
└──────────────────────────────────────────────────────────────────┘
```

### `civ-core`

- Aucune dépendance à Horizon, Minecraft, un moteur de rendu, un client HTTP ou un LLM.
- Types de données, règles, validation, structures de simulation, RNG, sauvegarde.
- API stable et petite : charger et valider un scénario, sérialiser un état, produire une sauvegarde.
- La session de `civ-horizon` avance l'horloge ; les systèmes de jeu actifs sont implémentés à l'étape 3 dans `sim/`.
- Toutes les décisions sont reproductibles.
- Les erreurs sont typées et ne paniquent pas sur une donnée utilisateur.

### `civ-horizon`

- Dépend de `civ-core` uniquement.
- Définit un trait `Host` pour les effets de bord : envoyer un message, jouer un son, faire apparaître une particule, ouvrir un écran, demander une sauvegarde, envoyer un message réseau.
- Contient les commandes `/civ`, la session, les modèles d'UI, les messages réseau, le trait `ExplosionEffects`, les écrans de mod, la géométrie de rendu 3D, le directeur cinématique et la synchronisation réseau.
- Peut être testé avec un `Host` factice, sans Horizon.
- Ne connaît aucun nom d'API Horizon.

### `glue.rs`

- Module injecté dans `crates/horizon-client/src/game.rs` via `horizon-mixin.toml`.
- Implémente `Host` avec les helpers Horizon vérifiés.
- Reçoit les hooks, les convertit en commandes ou en appels de session.
- Ne contient pas de règles de jeu : il ne décide pas qu'une guerre est juste, il transmet.
- Les fichiers `register_commands.rs`, `perform_command.rs`, `hud.rs`, etc. sont de petites injections qui appellent `crate::game::civ_glue`.

## Flux cible d'une partie

```text
1. Chargement du pack de données
   manifest.json -> schémas -> validation métier -> registres par ID
   (GameData JSON chargé et validé à l'étape 3 ; data-pack complet à l'étape 5)

2. Création ou chargement d'une partie
   scénario initial + graine -> WorldState + NationState + DiplomacyState
   ou sauvegarde JSON -> vérification -> migration -> état

3. Boucle serveur
   native_server_tick -> Session::advance_ticks + systèmes actifs
     - économie et population
     - IA (diplomatie, budget, guerre)
     - événements
     - batailles, occupations, paix
     - révolutions
     - missiles en vol
   -> snapshot du tick

4. Diffusion
   serveur -> clients : messages réseau par `Host::send_net`
   client -> serveur : commandes via le protocole de l'adaptateur
   serveur autoritaire : toute commande est validée avant application

5. Présentation
   HUD, carte, fiches, alertes, cinématiques (étape 4)
   -> lecture seule des snapshots ; aucune modification de l'état par le client
```

## Déterminisme

Règles applicables à `civ-core` :

- Aucun `HashMap` itéré sans tri préalable.
- Aucun `Instant::now`, `SystemTime`, `rand::thread_rng` ou framerate dans la simulation.
- RNG SplitMix64 à compteur, avec état sérialisé (`seed`, `counter`, `stream`).
- Flux séparés implémentés dans `sim/` pour `ai`, `diplomacy`, `war`, `revolution`, `events`, `missiles`.
- Tie-break systématique par identifiant stable.
- Les commandes sont appliquées dans l'ordre de réception serveur, jamais dans l'ordre d'un ensemble non ordonné.
- Les sauvegardes enregistrent l'état du RNG, pas seulement la graine.

## Autorité et multijoueur

- Le serveur est la seule autorité.
- Le client envoie des intentions (`/civ ...`, clics d'écran) et reçoit des snapshots.
- Le client ne modifie jamais directement une nation, une province, une guerre ou un missile.
- Les commandes admin sont vérifiées côté serveur selon le niveau de permission.
- Une partie solo utilise le même chemin serveur, avec un serveur local.
- Les messages réseau sont bornés en taille et validés ; un client malveillant ne doit pas pouvoir injecter un état arbitraire.

## Données éditables

- Les données de référence vivront dans `data/civ/` (JSON/TOML) et seront embarquées à la compilation (étape 5).
- Au premier lancement, elles seront matérialisées dans `<instance>/config/civ/`.
- Les fichiers externes seront prioritaires ; un override invalide sera refusé avec un message clair.
- Les schémas JSON sont la première barrière ; `civ-core` applique ensuite les règles métier (IDs uniques, références existantes, polygones simples, cohérence des frontières, etc.).
- Aucun chargement ne dépend d'une API de contenu Horizon non vérifiée : l'adaptateur utilise `std::fs` et `native_instance_dir()`.

## Sauvegarde

- Fichier JSON sous `<world>/civ/saves/` via `native_world_dir()` (branchement étape 4).
- Écriture avec fichier temporaire, `sync_all()`, copie `.bak` et renommage.
- Checksum SHA-256 du JSON canonique, checksum exclu.
- Version de schéma, version du mod, commit Horizon et empreinte des données.
- Migration ordonnée ; une sauvegarde plus récente que le runtime est refusée, pas écrasée.
- Sauvegarde automatique configurable (étape 4) ; sauvegarde manuelle via `/civ save`.

## Ce que l'architecture refuse

- Un LLM ou un appel réseau pour l'IA.
- Une nation, un dirigeant ou une frontière réelle.
- Une dépendance de `civ-core` à Horizon.
- Une logique de jeu dans `glue.rs`.
- Une confiance au client pour l'état autoritaire.
- Une API Horizon inventée ou supposée stable sans preuve dans le commit épinglé.
