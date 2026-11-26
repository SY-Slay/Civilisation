# 03 — Format de sauvegarde

> **Statut :** le snapshot v1, le checksum SHA-256 canonique, la migration, la copie `.bak`, la validation métier, la reprise déterministe de l'horloge, l'autosave atomique, l'outil hors-ligne `tools/migrate_save.py` et les tests de non-régression multi-annuels sont intégralement implémentés et validés.

## Emplacement

```text
<répertoire du monde>/civ/saves/<save_id>.json
<répertoire du monde>/civ/saves/autosave.json
<répertoire du monde>/civ/saves/<save_id>.json.bak
```

Le répertoire du monde est obtenu via `native_world_dir()` côté serveur. Le client ne lit ni n'écrit les sauvegardes ; il reçoit un snapshot par le réseau.

## Format

- JSON UTF-8, lisible et éditable.
- Une sauvegarde peut être compressée au repos en `.json.gz`, mais le schéma décrit le JSON **décompressé**. Le checksum porte sur le JSON canonique décompressé.
- Aucun format binaire opaque au niveau 1.
- Les gros mondes peuvent dépasser plusieurs mégaoctets ; la compression est recommandée pour les sauvegardes automatiques, pas obligatoire.

## Structure de haut niveau

```text
{
  schema_version,
  save_id,                  UUID v4 fourni par l'adaptateur
  created_at, updated_at,   ISO-8601 UTC
  compatibility: {          jeu, mod, commit Horizon, empreinte du scénario
    game_version,
    mod_version,
    horizon_revision,
    data_fingerprint,
    schema_version
  },
  scenario_id,
  seed,
  tick,
  date,
  start_date?,              date initiale du scénario (optionnel, rétrocompatible)
  difficulty?,              difficulté initiale (optionnel)
  ticks_per_day?,           horloge de simulation (optionnel)
  world,                    monde complet (provinces, frontières, régions)
  nations[],                état complet des nations
  diplomacy,                relations, traités, organisations...
  wars[],                   guerres, fronts, batailles, armées
  revolutions[],
  armies[],                 armées hors guerre ou en réserve
  missiles_in_flight[],
  event_state: { active[], cooldowns[], fired_counts[], pending[] },
  rng: { algorithm, seed, counter, stream },
  checksum,                 SHA-256 hex
  metadata: { save_name, player_nation, playtime_seconds, notes, multiplayer, host }
}
```

Le schéma de référence est `schemas/save.schema.json`.

## Version 1 : snapshot complet

La version 1 privilégie l'autonomie :

- le monde et les nations sont inclus dans la sauvegarde ;
- une sauvegarde peut être rechargée même si le scénario d'origine a changé ;
- l'empreinte du scénario permet de détecter un scénario modifié ; l'empreinte globale du data-pack viendra à l'étape 5 ;
- une migration peut décider de recharger les données statiques ou de conserver le snapshot ;
- `start_date`, `difficulty` et `ticks_per_day` sont optionnels : les anciennes sauvegardes restent lisibles, et les nouvelles conservent l'horloge et l'IA exactes après rechargement ;
- `event_state.pending` conserve les effets différés avec leur tick d'application et leur nation cible.

À l'étape 3, `data_fingerprint` est encore le SHA-256 du scénario canonique chargé. Le `GameData` JSON (technologies, missiles, événements) est validé séparément par `civ-core`. Le chargeur complet de data-pack et son empreinte globale restent à l'étape 5 ; le champ ne changera pas de format.

Optimisation future possible : séparer l'état statique (scénario) de l'état dynamique (provinces, nations, guerres) pour réduire la taille. Ce n'est pas nécessaire pour la version 1.

## Checksum

### Canonicalisation

Avant le calcul :

1. supprimer le champ `checksum` ;
2. trier les clés de tous les objets par ordre lexical (`serde_json::Map` est un `BTreeMap` dans la configuration du projet) ;
3. utiliser des tableaux dans l'ordre du jeu (les tableaux ne sont pas triés) ;
4. encoder en UTF-8 sans BOM ;
5. supprimer les espaces insignifiants (`serde_json::to_string`) ;
6. normaliser les nombres : entiers sans `.0`, flottants avec une représentation décimale stable ;
7. refuser `NaN` et `Infinity` (sérialisation `serde_json`).

Le checksum est `SHA-256(JSON canonique)`, en hexadécimal minuscule.

### Vérification

- Au chargement, vérifier le checksum du document **d'origine**, avant migration.
- Si le checksum diffère, refuser la sauvegarde et proposer `.bak`.
- Un fichier édité à la main doit être re-scellé par un outil officiel (`tools/migrate_save.py` ou commande admin) ; sinon il est considéré comme corrompu.
- Après une migration qui modifie le document, le checksum est recalculé avant de rendre la sauvegarde utilisable. Le fichier d'origine reste inchangé tant que l'adaptateur ne l'a pas réécrit.
- Le checksum n'est pas une signature cryptographique : il détecte la corruption et les modifications accidentelles, pas un adversaire malveillant.

## Écriture atomique

1. vérifier le checksum courant ;
2. écrire dans `<save_id>.civ.tmp` ;
3. flush et `sync_all()` ;
4. renommer l'ancien fichier en `.json.bak` ;
5. renommer le temporaire vers le nom final ;
6. si le renommage final échoue, restaurer le `.bak`.

Jamais d'écriture directe dans le fichier final. Jamais de suppression du `.bak` avant que le nouveau fichier soit lisible. L'implémentation actuelle est dans `civ-core::save::Save::write_to_path`.

## Sauvegarde automatique

- `config.simulation.autosave_days` : intervalle en jours fictifs.
- `0` désactive l'autosave.
- L'autosave écrit `autosave.json` et conserve un `.bak`.
- Une sauvegarde manuelle via `/civ save [nom]` écrit un fichier horodaté.
- Une sauvegarde est refusée si une autre écriture est en cours.

## Chargement

1. lire le fichier ;
2. parser le JSON ;
3. désérialiser la version courante ;
4. vérifier `schema_version` et la version du jeu ;
5. vérifier l'identité (`save_id` UUID v4, nom, dates, `ticks_per_day` s'il est présent) ;
6. vérifier le checksum du document d'origine ;
7. appliquer les migrations connues ;
8. redésérialiser le document migré, vérifier de nouveau l'identité, puis re-sceler ;
9. appeler `Save::validate_payload` (via `SaveFile` ou `Session`) pour rejouer les invariants métier ;
10. restaurer `start_date`, `difficulty`, `ticks_per_day`, l'état RNG et les effets différés ;
11. remplacer l'état courant seulement si tout est valide.

La validation JSON Schema des données et des sauvegardes est effectuée en CI sur les fixtures ; les règles métier sont appliquées par `civ-core::validation`. En cas d'échec, l'état courant reste intact et l'erreur indique l'étape fautive.

## Migration

- Chaque migration est une fonction pure `version N -> N+1`, documentée et testée.
- Une sauvegarde plus récente que le runtime est refusée : message clair, aucune tentative de lecture partielle.
- Une sauvegarde plus ancienne est migrée en mémoire, re-scellée, puis l'adaptateur peut l'écrire sous un nouveau nom (`<save_id>.migrated.json`) avant de l'utiliser.
- La version 1 est la première version publique. La seule migration active renomme l'algorithme RNG de brouillon `splitmix64_pcg_v1` vers `splitmix64_counter_v1`.
- Les migrations ne doivent pas dépendre de l'horloge, du réseau ou d'un LLM.
- Les tests de migration conservent des fixtures de chaque version.

## Compatibilité

La sauvegarde enregistre :

- `game_version` : `26.3` ;
- `mod_version` : version sémantique du mod ;
- `horizon_revision` : commit de 40 caractères ;
- `data_fingerprint` : SHA-256 du scénario canonique chargé (l'empreinte du data-pack complet viendra à l'étape 5) ;
- `schema_version` : version du schéma de sauvegarde.

À l'étape 3, `check_compatibility` refuse une version de schéma ou de jeu différente. Les champs `mod_version`, `horizon_revision` et `data_fingerprint` sont enregistrés et exposés ; l'adaptateur de l'étape 4 décidera s'il demande confirmation ou applique une migration de données. Il ne charge jamais silencieusement un pack incompatible.

## Multijoueur

- Seul le serveur écrit les sauvegardes.
- Le client peut demander `/civ save` si la permission l'autorise ; le serveur exécute et répond.
- Une sauvegarde en multijoueur contient `metadata.multiplayer = true` et éventuellement `metadata.host`.
- Les commandes client ne sont jamais appliquées directement à la sauvegarde.

## Corruption et récupération

- Si le fichier principal est invalide, proposer le `.bak`.
- Si le `.bak` est invalide, refuser et conserver les deux fichiers pour diagnostic.
- Ne jamais écraser automatiquement un fichier corrompu par une nouvelle partie.
- Un journal d'erreurs peut être écrit dans `<world>/civ/logs/` sans données personnelles.

## Pourquoi pas d'attachement persistant au niveau 1

Les attachments Horizon sont utiles pour de petits pointeurs, mais exigent un codec et un format stable. La version 1 utilise uniquement des fichiers JSON sous le répertoire du monde :

- plus simple à sauvegarder, inspecter et migrer ;
- pas de dépendance à un codec interne ;
- pas de limite de taille imposée par un attachment.

Un attachment pourra être ajouté plus tard comme index rapide (`save_id`, `tick`, `checksum`) sans remplacer les fichiers.
