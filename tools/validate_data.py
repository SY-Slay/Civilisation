#!/usr/bin/env python3
"""Validation métier complète du pack de données Civilisations.

Vérifie :
1. Conformité JSON Schema (Draft 2020-12) de tous les fichiers du pack.
2. Intégrité référentielle croisée (nations, provinces, registres, technologies, missiles, relations).
3. Règles géométriques des frontières (irrégularité, absence de ligne droite > 160 blocs, réciprocité, simple polygone).
4. Politique de contenu (aucun terme ou état réel, 100% fictionnel).
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012

ROOT = Path(__file__).resolve().parent.parent
SCHEMAS_DIR = ROOT / "schemas"
DATA_DIR = ROOT / "data" / "civ"

FORBIDDEN_REAL_NAMES = {
    "france", "germany", "allemagne", "russia", "russie", "china", "chine",
    "usa", "united states", "états-unis", "united_states", "britain", "uk",
    "italy", "italie", "spain", "espagne", "japan", "japon", "paris", "berlin",
    "moscow", "moscou", "washington", "london", "londres", "beijing", "tokyo",
}


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
        schemas[path.name] = schema
        resources.append((schema_id, Resource.from_contents(schema, default_specification=DRAFT202012)))
    return schemas, Registry().with_resources(resources)


def validate_schema(instance: object, schema: object, registry: Registry, label: str) -> list[str]:
    validator = Draft202012Validator(schema, registry=registry, format_checker=FormatChecker())
    errors = sorted(validator.iter_errors(instance), key=lambda error: list(error.path))
    res = []
    for error in errors[:10]:
        loc = "/".join(str(p) for p in error.path) or "<root>"
        res.append(f"  [{label}] {loc}: {error.message}")
    return res


def cross_product(p1, p2, p3):
    return (p2[0] - p1[0]) * (p3[1] - p1[1]) - (p2[1] - p1[1]) * (p3[0] - p1[0])


def is_simple_polygon(pts: list[list[float]]) -> bool:
    n = len(pts)
    if n < 3:
        return False
    # Check no non-consecutive edges intersect
    def ccw(A, B, C):
        return (C[1] - A[1]) * (B[0] - A[0]) > (B[1] - A[1]) * (C[0] - A[0])

    def intersect(A, B, C, D):
        return ccw(A, C, D) != ccw(B, C, D) and ccw(A, B, C) != ccw(A, B, D)

    for i in range(n):
        a1, a2 = pts[i], pts[(i + 1) % n]
        for j in range(i + 2, n):
            if i == 0 and j == n - 1:
                continue
            b1, b2 = pts[j], pts[(j + 1) % n]
            if intersect(a1, a2, b1, b2):
                return False
    return True


def check_content_policy(data: object, path: str, errors: list[str]):
    if isinstance(data, dict):
        for k, v in data.items():
            check_content_policy(v, f"{path}.{k}", errors)
    elif isinstance(data, list):
        for idx, item in enumerate(data):
            check_content_policy(item, f"{path}[{idx}]", errors)
    elif isinstance(data, str):
        lower = data.lower()
        for forbidden in FORBIDDEN_REAL_NAMES:
            if forbidden in lower:
                errors.append(f"Politique de contenu violée dans {path} : mention de '{forbidden}' détectée")


def validate_all() -> int:
    print("=== Validation des schémas et règles métier de Civilisations ===")
    schemas, registry = load_schemas()
    errors: list[str] = []

    # 1. Validation du manifeste
    manifest_path = DATA_DIR / "manifest.json"
    if not manifest_path.exists():
        print("ERREUR : manifest.json absent dans data/civ/", file=sys.stderr)
        return 1

    manifest = read_json(manifest_path)
    errs = validate_schema(manifest, schemas["data-pack.schema.json"], registry, "manifest.json")
    errors.extend(errs)

    # 2. Validation de chaque fichier déclaré dans le manifeste
    schema_map = {
        "cultures": "culture.schema.json",
        "religions": "religion.schema.json",
        "ideologies": "ideology.schema.json",
        "governments": "government.schema.json",
        "resources": "resource.schema.json",
        "technologies": "technology.schema.json",
        "units": "unit.schema.json",
        "missiles": "missile.schema.json",
        "events": "event.schema.json",
        "names": "names.schema.json",
        "localization": "localization.schema.json",
        "nations": "nation.schema.json",
        "provinces": "province.schema.json",
        "scenarios": "scenario.schema.json",
    }

    content_dict = manifest.get("content", {})
    files_checked = 0

    cultures_ids = set()
    religions_ids = set()
    ideologies_ids = set()
    governments_ids = set()
    resources_ids = set()
    technologies_ids = set()
    units_ids = set()
    missiles_ids = set()

    for key, schema_name in schema_map.items():
        rel_paths = content_dict.get(key, [])
        for rel in rel_paths:
            fpath = DATA_DIR / rel
            if not fpath.exists():
                errors.append(f"Fichier déclaré introuvable : {rel}")
                continue
            obj = read_json(fpath)
            errs = validate_schema(obj, schemas[schema_name], registry, rel)
            errors.extend(errs)
            files_checked += 1

            # Collecte des identifiants
            if key == "cultures":
                cultures_ids.add(obj["id"])
            elif key == "religions":
                religions_ids.add(obj["id"])
            elif key == "ideologies":
                ideologies_ids.add(obj["id"])
            elif key == "governments":
                governments_ids.add(obj["id"])
            elif key == "resources":
                resources_ids.add(obj["id"])
            elif key == "technologies":
                technologies_ids.add(obj["id"])
            elif key == "units":
                units_ids.add(obj["id"])
            elif key == "missiles":
                missiles_ids.add(obj["id"])

            # Vérification politique de contenu
            check_content_policy(obj, rel, errors)

    print(f"Fichiers validés contre les schémas JSON : {files_checked}")

    # 3. Validation approfondie du scénario
    scenarios = content_dict.get("scenarios", [])
    if not scenarios:
        errors.append("Aucun scénario dans le manifeste")
    else:
        scen_path = DATA_DIR / scenarios[0]
        scenario = read_json(scen_path)
        world = scenario["world"]
        provinces = world["provinces"]
        borders = world["borders"]
        nations = scenario["nations"]
        diplomacy = scenario["diplomacy"]

        prov_map = {p["id"]: p for p in provinces}
        border_map = {b["id"]: b for b in borders}

        # Unicité
        if len(prov_map) != len(provinces):
            errors.append("Doublons détectés dans les identifiants de provinces")
        if len(border_map) != len(borders):
            errors.append("Doublons détectés dans les identifiants de frontières")

        # Vérification des frontières
        border_ref_counts = {b["id"]: 0 for b in borders}
        for p in provinces:
            pid = p["id"]
            # Polygone simple et aire
            poly = p["polygon"]
            if not is_simple_polygon(poly):
                errors.append(f"Province {pid} : polygone auto-intersectant ou invalide")
            if p["area_blocks"] < 1024:
                errors.append(f"Province {pid} : surface trop petite ({p['area_blocks']} < 1024)")

            # Voisins et bordures
            for bid in p["border_ids"]:
                if bid not in border_map:
                    errors.append(f"Province {pid} référence frontière inconnue {bid}")
                else:
                    border_ref_counts[bid] += 1

            for nid in p["neighbors"]:
                if nid not in prov_map:
                    errors.append(f"Province {pid} référence voisin inconnu {nid}")
                else:
                    if pid not in prov_map[nid]["neighbors"]:
                        errors.append(f"Non-réciprocité de voisinage entre {pid} et {nid}")

            # Registres des provinces
            if p["culture"]["primary"] not in cultures_ids:
                errors.append(f"Province {pid} : culture inconnue {p['culture']['primary']}")
            if p["religion"]["primary"] not in religions_ids:
                errors.append(f"Province {pid} : religion inconnue {p['religion']['primary']}")
            for res in p["resources"]:
                if res["resource"] not in resources_ids:
                    errors.append(f"Province {pid} : ressource inconnue {res['resource']}")

        # Réciprocité stricte des frontières partagées (exactement 2 références)
        for bid, count in border_ref_counts.items():
            if count != 2:
                errors.append(f"Frontière {bid} référencée {count} fois au lieu de 2")

        # Test d'irrégularité des frontières (pas de segment droit > 160 blocs)
        for b in borders:
            pts = b["points"]
            if len(pts) < 3:
                errors.append(f"Frontière {b['id']} trop simpliste (< 3 sommets)")
            for k in range(len(pts) - 1):
                seg_len = math.hypot(pts[k+1][0] - pts[k][0], pts[k+1][1] - pts[k][1])
                if seg_len > 160.0:
                    errors.append(f"Frontière {b['id']} : segment trop rectiligne ({seg_len:.1f} > 160 blocs)")

        # Vérification des nations
        nation_map = {n["id"]: n for n in nations}
        for n in nations:
            nid = n["id"]
            if n["culture"] not in cultures_ids:
                errors.append(f"Nation {nid} : culture inconnue {n['culture']}")
            if n["religion"] not in religions_ids:
                errors.append(f"Nation {nid} : religion inconnue {n['religion']}")
            if n["ideology"] not in ideologies_ids:
                errors.append(f"Nation {nid} : idéologie inconnue {n['ideology']}")
            if n["government"] not in governments_ids:
                errors.append(f"Nation {nid} : gouvernement inconnu {n['government']}")
            if n["capital"] not in n["provinces"]:
                errors.append(f"Nation {nid} : capitale absente de ses provinces")
            for pid in n["provinces"]:
                if pid not in prov_map:
                    errors.append(f"Nation {nid} possède province inconnue {pid}")
                elif prov_map[pid]["owner"] != nid:
                    errors.append(f"Nation {nid} possède {pid} mais owner = {prov_map[pid]['owner']}")

            for tech in n["technology"]["researched"]:
                if tech not in technologies_ids:
                    errors.append(f"Nation {nid} a recherché technologie inconnue {tech}")

            for m_id in n["military"].get("missile_stockpile", {}):
                if m_id not in missiles_ids:
                    errors.append(f"Nation {nid} a en stock missile inconnu {m_id}")

        # Relations diplomatiques : a < b obligatoire
        for rel in diplomacy.get("relations", []):
            if rel["a"] not in nation_map or rel["b"] not in nation_map:
                errors.append(f"Relation entre nations inconnues : {rel['a']} - {rel['b']}")
            if rel["a"] >= rel["b"]:
                errors.append(f"Ordre lexical non respecté dans relation : {rel['a']} >= {rel['b']}")

    # Bilan
    if errors:
        print(f"\nÉCHEC : {len(errors)} erreur(s) détectée(s) :", file=sys.stderr)
        for err in errors[:40]:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print("\nSUCCÈS : 100% des fichiers et règles métier validés sans erreur.")
    print("  - Tous les schémas JSON Draft 2020-12 validés.")
    print("  - Intégrité référentielle complète.")
    print("  - Frontières irrégulières, sans segment droit excessif, réciproques.")
    print("  - Politique de contenu 100% respectée (aucun nom réel, pure fiction).")
    return 0


if __name__ == "__main__":
    sys.exit(validate_all())
