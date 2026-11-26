# Fixtures de validation des schémas

Ces fichiers ne sont **pas** du contenu de production. Ils prouvent que les schémas JSON et leurs `$ref` fonctionnent ensemble.

## Contenu

- `valid/` : 22 objets minimaux valides, dont `scenario.json`, `save.json` et `game-data.json`.
- `invalid/` : non créé à l'étape 3. Les rejets métier sont testés directement dans `civ-core` (identifiants, voisinage, capitale, checksum, migration, données de jeu) plutôt que par des fixtures JSON Schema séparées.

## Validation

Prérequis : Python 3.10+ et `jsonschema` 4.26 ou plus récent.

```powershell
python tools/validate_schemas.py
```

Le contrôle effectué :

1. charger les 24 fichiers `schemas/*.schema.json` ;
2. vérifier chaque schéma avec `Draft202012Validator.check_schema` ;
3. construire un registre `referencing` indexé par `$id` ;
4. résoudre tous les `$ref` croisés ;
5. valider chaque fixture contre son schéma avec `FormatChecker`.

Résultat attendu :

```text
24 schémas valides, 22 fixtures valides, tous les $ref résolus
```

Un export réel de sauvegarde produit par `civ-horizon` a également été validé contre `save.schema.json` avec `FormatChecker`.

## Limites

- Le monde de `world.json`, `scenario.json` et `save.json` est une grille jitterée de 8 provinces. Il ne respecte pas les règles de production sur l'irrégularité des frontières.
- Les IDs, nations, dirigeants et événements sont inventés et minimalistes.
- Les fixtures ne remplacent pas les tests métier de `civ-core` : unicité globale, réciprocité des voisins, capitale dans la nation, checksum canonique, migration et rechargement sont couverts par `cargo test`.
