# Civilisations

Mod de géopolitique solo pour **Horizon 26.3** : nations fictives, diplomatie, révolutions, guerres, missiles et cinématiques.

> **Statut : étapes 0 à 6 livrées pour le périmètre local vérifié.** L'étape 0 a prouvé le flux natif avec `/civ ping`. L'étape 1 a livré l'arborescence, 24 schémas JSON et 22 fixtures. L'étape 2 fournit `civ-core` (simulation pure) et `civ-horizon` (adaptateur testable). L'étape 3 ajoute la boucle géopolitique déterministe et les commandes `/civ`. L'étape 4 livre l'adaptateur Horizon (`glue.rs`, 18 hooks natifs, rendu 3D, cinématique et packaging `.hmod`), vérifié en monde solo avant le correctif de carte actuel. L'étape 5 livre le pack de données de production complet (`data/civ/`) avec le scénario « Arc des Frontières », 4 nations, 16 provinces, registres, technologies, unités, missiles, événements, localisation FR/EN, assets originaux et l'outil de validation métier `tools/validate_data.py`. L'étape 6 achève la finition : tests golden 5 ans déterministes, reprise après sauvegarde, tests de migration, outil hors-ligne `tools/migrate_save.py`, note d'état sur le support serveur dédié (non vérifié), guide de création de packs et licences officielles.

## Ce qui existe

- `civ-ping/` : ancien bundle natif `.hnmod` de preuve, avec `/civ ping`.
- `PROJECT_TREE.md` : arborescence complète et statut de chaque fichier.
- `LICENSE` : textes officiels des licences MPL-2.0 (code) et CC-BY-SA-4.0 (données/assets).
- `schemas/` : 24 schémas JSON Schema Draft 2020-12, dont l'agrégat `game-data.schema.json`.
- `tests/schemas/valid/` : 22 fixtures valides, dont une sauvegarde et un jeu de données.
- `data/civ/` : data-pack de production complet (scénario « Arc des Frontières », 4 nations, 16 provinces, registres complets de cultures, religions, idéologies, gouvernements, ressources, technologies, unités, missiles, événements et localisation FR/EN).
- `assets/civ/` : assets originaux sous licence CC-BY-SA-4.0 (icône PNG, drapeaux PNG des 4 nations, textures, localisation).
- `native/civ/mixins/civ/civ-core/` : IDs typés, temps, RNG déterministe, géométrie, monde, nations, diplomatie, guerre, révolutions, missiles, événements, validation, chargeur de data-pack (`load_data_pack`), sauvegarde SHA-256, migration, et simulation géopolitique.
- `native/civ/mixins/civ/civ-horizon/` : trait `Host`, session, commandes `/civ`, sauvegarde atomique, effets d'explosion, modèles d'UI et widgets, rendu 3D des frontières et missiles, directeur cinématique et protocoles réseau.
- `native/civ/` : manifeste `horizon-native.json`, `horizon-mixin.toml`, `glue.rs` et les 18 adaptateurs de hooks d'injection Horizon.
- `tools/package_native.py` : assembleur reproductible du bundle natif `civ.hmod` (code, données et assets).
- `tools/validate_schemas.py` : validateur Draft 2020-12 des 24 schémas et 22 fixtures.
- `tools/validate_data.py` : validateur métier du pack de données (schémas, intégrité référentielle, géométrie irrégulière et politique de contenu 100% fictionnelle).
- `tools/migrate_save.py` : outil d'inspection et de migration hors ligne des sauvegardes JSON.
- `tools/build_data_pack.py` : générateur déterministe du pack de production.
- `docs/` : politique de contenu, architecture, modèle de données, format de sauvegarde, commandes, APIs Horizon vérifiées, état et limites du support serveur dédié, feuille de route, guide de création de packs tiers.

## Architecture en une phrase

`civ-core` contient toute la simulation pure et déterministe ; `civ-horizon` contient l'adaptation testable ; `glue.rs` est la seule couche qui connaît les APIs Horizon.

## Compilation et tests

Prérequis : Rust 1.80 ou plus récent.

```powershell
cargo test --locked -p civ-core -p civ-horizon
cargo clippy -p civ-core -p civ-horizon --all-targets -- -D warnings
cargo fmt --all -- --check
```

Clippy et le formateur de code passent avec 0 avertissement. Résultat final des tests :

```text
civ-core unitaires       : 39 tests verts
civ-core datapack        :  2 tests verts
civ-core géopolitique    :  4 tests verts
civ-core golden (5 ans)  :  2 tests verts
civ-core migration       :  5 tests verts
civ-horizon unitaires    : 18 tests verts
civ-horizon actions      :  3 tests verts
civ-horizon session      :  7 tests verts
doc-tests                :  0 test
launcher console         :  3 tests verts
total civ-core/horizon   : 80 tests verts
total tous les packages : 83 tests verts
```

Les tests vérifient notamment :
- Le déterminisme strict sur 5 ans fictifs (checksums scellés et journaux d'événements rigoureusement identiques entre deux simulations complètes).
- La reprise de partie depuis une sauvegarde (reprise à l'année 1 aboutissant à l'identique de la partie ininterrompue à l'année 3).
- La migration des anciennes sauvegardes (rejet v0, migration RNG vers `splitmix64_counter_v1`, validation et re-scellement).
- L'intégrité de la simulation officielle sur le pack « Arc des Frontières ».
- Deux sessions identiques produisant le même checksum, et une sauvegarde se rechargeant à l'identique.
- La persistance atomique avec copie de secours `.bak`.

## Validation des schémas et des données

```powershell
python tools/validate_schemas.py
python tools/validate_data.py
```

Résultats :
- 24 schémas Draft 2020-12 validés, 22 fixtures conformes.
- 67 fichiers du pack de données audités : 0 erreur de schéma, intégrité référentielle complète, 24 frontières réciproques sans ligne droite > 160 blocs, et 100% fictionnel.

## Documentation

- `docs/00-content-policy.md` — fiction, licences, frontières, IA.
- `docs/01-architecture.md` — séparation des couches.
- `docs/02-data-model.md` — schémas et modèle de données.
- `docs/03-save-format.md` — JSON, checksum SHA-256, migration.
- `docs/04-commands.md` — commandes `/civ`, syntaxe, autorité.
- `docs/05-horizon-api.md` — hooks vérifiés et limites.
- `docs/06-dedicated-server.md` — état non vérifié du support dédié, comportements présents et validations manquantes.
- `docs/07-roadmap.md` — étapes 0 à 6 complètes.
- `docs/08-pack-creation-guide.md` — guide de création, règles géométriques et validation de packs de données tiers.

## Licence

- Code : Mozilla Public License 2.0 (MPL-2.0). Voir `LICENSE`.
- Données & Assets : Creative Commons Attribution-ShareAlike 4.0 International (CC-BY-SA-4.0). Voir `LICENSE` et `assets/civ/README.md`.

Aucune nation, aucun dirigeant et aucun territoire réel ne sont représentés.
