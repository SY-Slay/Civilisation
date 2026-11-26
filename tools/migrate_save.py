#!/usr/bin/env python3
"""Outil d'inspection et de migration hors ligne des sauvegardes de Civilisations.

Vérifie la compatibilité d'une sauvegarde, migre les versions de schéma obsolètes
et valide l'intégrité de l'instantané contre schemas/save.schema.json.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012

ROOT = Path(__file__).resolve().parent.parent
SCHEMAS_DIR = ROOT / "schemas"
CURRENT_SAVE_VERSION = 1
CANONICAL_RNG = "splitmix64_counter_v1"


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_save_validator() -> tuple[Draft202012Validator, Registry]:
    schemas: dict[str, object] = {}
    resources: list[tuple[str, Resource]] = []
    for path in sorted(SCHEMAS_DIR.glob("*.schema.json")):
        schema = read_json(path)
        schema_id = schema.get("$id")
        schemas[path.name] = schema
        resources.append((schema_id, Resource.from_contents(schema, default_specification=DRAFT202012)))
    reg = Registry().with_resources(resources)
    validator = Draft202012Validator(schemas["save.schema.json"], registry=reg, format_checker=FormatChecker())
    return validator, reg


def migrate_save_data(save: dict) -> tuple[dict, list[str]]:
    """Applique les migrations connues au document JSON de sauvegarde."""
    changes = []
    version = save.get("schema_version", 0)

    if version == 0:
        raise ValueError("Version de sauvegarde 0 non supportée")
    if version > CURRENT_SAVE_VERSION:
        raise ValueError(f"Version de sauvegarde future {version} non supportée")

    # Migration v1 : renommage de l'algorithme RNG brouillon
    rng = save.get("rng")
    if isinstance(rng, dict):
        algo = rng.get("algorithm")
        if algo == "splitmix64_pcg_v1":
            rng["algorithm"] = CANONICAL_RNG
            changes.append(f"RNG algorithm migré de 'splitmix64_pcg_v1' vers '{CANONICAL_RNG}'")

    # Compatibilité schema_version
    compat = save.get("compatibility")
    if isinstance(compat, dict) and compat.get("schema_version") is None:
        compat["schema_version"] = CURRENT_SAVE_VERSION
        changes.append("compatibility.schema_version initialisé à 1")

    return save, changes


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="Fichier de sauvegarde JSON à inspecter/migrer")
    parser.add_argument("--out", type=Path, help="Fichier de sortie (si absent, mode vérification)")
    parser.add_argument("--dry-run", action="store_true", help="Affiche les migrations sans écrire")
    args = parser.parse_args()

    if not args.input.exists():
        print(f"ERREUR : fichier introuvable : {args.input}", file=sys.stderr)
        return 1

    try:
        data = read_json(args.input)
    except Exception as e:
        print(f"ERREUR lors de la lecture JSON : {e}", file=sys.stderr)
        return 1

    print(f"Inspection de la sauvegarde : {args.input}")
    print(f"  - Save ID      : {data.get('save_id')}")
    print(f"  - Version      : {data.get('schema_version')}")
    print(f"  - Horloge      : Tick {data.get('tick')}, Date {data.get('date')}")
    print(f"  - Checksum     : {data.get('checksum')}")

    try:
        migrated, changes = migrate_save_data(data)
    except ValueError as e:
        print(f"ERREUR de migration : {e}", file=sys.stderr)
        return 1

    if changes:
        print(f"  - {len(changes)} migration(s) appliquée(s) :")
        for ch in changes:
            print(f"      * {ch}")
    else:
        print("  - Sauvegarde déjà au format cible le plus récent.")

    # Validation de schéma
    validator, _ = load_save_validator()
    errors = list(validator.iter_errors(migrated))
    if errors:
        print(f"ATTENTION : {len(errors)} erreur(s) de schéma détectée(s) :", file=sys.stderr)
        for err in errors[:10]:
            loc = "/".join(str(p) for p in err.path) or "<root>"
            print(f"  [{loc}] {err.message}", file=sys.stderr)
        return 1

    print("  - Schéma JSON Draft 2020-12 validé avec succès.")

    if args.out and not args.dry_run:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(migrated, indent=2, ensure_ascii=False), encoding="utf-8")
        print(f"Sauvegarde migrée enregistrée dans : {args.out}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
