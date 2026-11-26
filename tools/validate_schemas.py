#!/usr/bin/env python3
"""Valide les schémas JSON et les fixtures de Civilisations.

Le script ne fait aucun appel réseau. Il vérifie les 23 schémas Draft 2020-12,
résout les `$ref` croisés via leur `$id`, puis valide les 21 fixtures.
Il peut aussi valider un ou plusieurs exports de sauvegarde avec `--save`.

Prérequis : Python 3.10+ et `jsonschema` 4.26+.
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
FIXTURES_DIR = ROOT / "tests" / "schemas" / "valid"


def read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def load_schemas() -> tuple[dict[str, object], Registry]:
    schemas: dict[str, object] = {}
    resources: list[tuple[str, Resource]] = []
    for path in sorted(SCHEMAS_DIR.glob("*.schema.json")):
        schema = read_json(path)
        if not isinstance(schema, dict):
            raise ValueError(f"{path.name} n'est pas un objet JSON")
        Draft202012Validator.check_schema(schema)
        schema_id = schema.get("$id")
        if not isinstance(schema_id, str) or not schema_id:
            raise ValueError(f"{path.name} n'a pas de $id")
        schemas[path.name] = schema
        resources.append(
            (schema_id, Resource.from_contents(schema, default_specification=DRAFT202012))
        )
    return schemas, Registry().with_resources(resources)


def validate(
    instance: object,
    schema: object,
    registry: Registry,
    label: str,
) -> bool:
    validator = Draft202012Validator(
        schema,
        registry=registry,
        format_checker=FormatChecker(),
    )
    errors = sorted(validator.iter_errors(instance), key=lambda error: list(error.path))
    if not errors:
        return True
    print(f"INVALIDE : {label}", file=sys.stderr)
    for error in errors[:30]:
        location = "/".join(str(part) for part in error.path) or "<racine>"
        print(f"  {location} : {error.message}", file=sys.stderr)
    if len(errors) > 30:
        print(f"  ... {len(errors) - 30} erreur(s) supplémentaire(s)", file=sys.stderr)
    return False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--save",
        action="append",
        default=[],
        metavar="FICHIER",
        help="export de sauvegarde à valider contre save.schema.json (répétable)",
    )
    args = parser.parse_args()

    schemas, registry = load_schemas()
    failures = 0
    fixture_count = 0

    for path in sorted(FIXTURES_DIR.glob("*.json")):
        schema_name = path.name.replace(".json", ".schema.json")
        if schema_name not in schemas:
            print(f"INVALIDE : {path.name} n'a pas de schéma {schema_name}", file=sys.stderr)
            failures += 1
            continue
        if not validate(read_json(path), schemas[schema_name], registry, path.name):
            failures += 1
        fixture_count += 1

    for raw_path in args.save:
        path = Path(raw_path)
        if not path.is_absolute():
            path = ROOT / path
        if not path.is_file():
            print(f"INVALIDE : sauvegarde introuvable : {path}", file=sys.stderr)
            failures += 1
            continue
        if not validate(read_json(path), schemas["save.schema.json"], registry, str(path)):
            failures += 1

    if failures:
        print(f"{failures} validation(s) en échec", file=sys.stderr)
        return 1

    print(
        f"{len(schemas)} schémas valides, {fixture_count} fixtures valides, "
        "tous les $ref résolus"
    )
    if args.save:
        print(f"{len(args.save)} sauvegarde(s) valide(s) contre save.schema.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
