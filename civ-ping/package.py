#!/usr/bin/env python3
"""Fabrique `civ.hnmod` à partir des sources de ce dossier.

Le fichier produit est un zip reproductible :
  horizon-native.json
  mixins/civ/horizon-mixin.toml
  mixins/civ/*.rs

Usage :
  python package.py
  python package.py --out C:\\chemin\\vers\\civ.hnmod
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parent
MOD_ID = "civ"
VERSION = "0.0.1"
GAME = "26.3"
HORIZON = "89f236e3ef8e8b554bef507216b86ca90b552213"
MIXINS = "a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab"
STAMP = (2026, 1, 1, 0, 0, 0)


def manifest() -> dict[str, object]:
    return {
        "schema": 1,
        "id": MOD_ID,
        "version": VERSION,
        "game": GAME,
        "horizon": HORIZON,
        "mixins": MIXINS,
    }


def bundle_files() -> list[Path]:
    bundle = ROOT / "mixins" / MOD_ID
    files = sorted(
        p for p in bundle.rglob("*") if p.is_file() and p.suffix in {".rs", ".toml"}
    )
    if not files:
        raise SystemExit(f"aucun fichier .rs/.toml sous {bundle}")
    for p in files:
        rel = p.relative_to(ROOT).as_posix()
        if not rel.startswith(f"mixins/{MOD_ID}/"):
            raise SystemExit(f"{rel}: chemin inattendu")
    return files


def check_manifest() -> None:
    path = ROOT / "mixins" / MOD_ID / "horizon-mixin.toml"
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    mod = data.get("mod", {})
    compat = data.get("compatibility", {})
    if mod.get("id") != MOD_ID:
        raise SystemExit(f"{path}: [mod].id doit être {MOD_ID!r}")
    if mod.get("version") != VERSION:
        raise SystemExit(f"{path}: [mod].version doit être {VERSION!r}")
    if compat.get("horizon_revision") != HORIZON:
        raise SystemExit(f"{path}: horizon_revision ne correspond pas à {HORIZON}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / f"{MOD_ID}.hnmod")
    args = parser.parse_args()

    check_manifest()
    payload = bundle_files()
    out = args.out.resolve()
    out.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        def add(name: str, data: bytes) -> None:
            info = zipfile.ZipInfo(name, STAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data)

        add("horizon-native.json", (json.dumps(manifest(), indent=2) + "\n").encode())
        for path in payload:
            add(path.relative_to(ROOT).as_posix(), path.read_bytes())

    digest = hashlib.sha256(out.read_bytes()).hexdigest()
    print(f"{out}")
    print(f"sha256 {digest}")
    print(f"{len(payload) + 1} entrées, {out.stat().st_size} octets")


if __name__ == "__main__":
    main()
