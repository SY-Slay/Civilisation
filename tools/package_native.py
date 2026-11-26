#!/usr/bin/env python3
"""Build Horizon's flat native mod project and reproducible `.hmod` package."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parent.parent
NATIVE_DIR = ROOT / "native" / "civ"
MIXIN_DIR = NATIVE_DIR / "mixins" / "civ"
ADAPTER_SRC = MIXIN_DIR / "civ-horizon" / "src"
CORE_SRC = MIXIN_DIR / "civ-core" / "src"
MOD_ID = "civ"
VERSION = "0.1.0"
GAME = "26.3"
HORIZON = "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0"
MIXINS = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
PROJECT_DIR = ROOT / "native" / "civ-project"
STAMP = (2026, 11, 16, 0, 0, 0)
CONTENT_SEGMENT_CHARS = frozenset(
    "abcdefghijklmnopqrstuvwxyz0123456789_.-"
)

HOOK_TARGETS = {
    "frame": "native_frame",
    "mods": "native_mods",
    "key": "native_key",
    "register_commands": "native_register_commands",
    "perform_command": "native_perform_command",
    "perform_console_command": "native_perform_console_command",
    "server_tick": "native_server_tick",
    "world_lifecycle": "native_world_lifecycle",
    "hud": "native_hud",
    "screen_render_post": "native_screen_render_post",
    "screen_mouse_button_pressed_pre": "native_screen_mouse_button_pressed_pre",
    "screen_key_pressed_pre": "native_screen_key_pressed_pre",
    "screen_char_typed_pre": "native_screen_char_typed_pre",
    "camera_pose": "native_camera_pose",
    "camera_shake": "native_camera_shake",
    "render_hand": "native_render_hand",
    "net_from_client": "native_net_from_client",
    "net_from_server": "native_net_from_server",
    "player_logged_in": "native_player_logged_in",
    "player_logged_out": "native_player_logged_out",
}


def toml_value(value: object) -> str:
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(item) for item in value) + "]"
    if isinstance(value, dict):
        fields = ", ".join(
            f"{key} = {toml_value(item)}" for key, item in value.items()
        )
        return "{ " + fields + " }"
    raise TypeError(f"Valeur TOML non prise en charge : {type(value).__name__}")


def table_lines(values: dict[str, object]) -> list[str]:
    return [f"{key} = {toml_value(value)}" for key, value in values.items()]


def source_manifest() -> tuple[str, dict[str, object]]:
    native_path = NATIVE_DIR / "horizon-native.json"
    mixin_path = MIXIN_DIR / "horizon-mixin.toml"
    if not native_path.is_file():
        raise SystemExit(f"Manifeste natif manquant : {native_path}")
    if not mixin_path.is_file():
        raise SystemExit(f"Manifeste mixin manquant : {mixin_path}")

    legacy_native = json.loads(native_path.read_text(encoding="utf-8"))
    legacy_mixins = tomllib.loads(mixin_path.read_text(encoding="utf-8"))
    if (
        legacy_native.get("id") != MOD_ID
        or legacy_native.get("version") != VERSION
        or legacy_native.get("game") != GAME
        or legacy_native.get("horizon") != HORIZON
        or legacy_native.get("mixins") != MIXINS
    ):
        raise SystemExit(f"Pins natifs incorrects dans {native_path}")
    if (
        legacy_mixins.get("mod", {}).get("id") != MOD_ID
        or legacy_mixins.get("mod", {}).get("version") != VERSION
        or legacy_mixins.get("compatibility", {}).get("horizon_revision") != HORIZON
    ):
        raise SystemExit(f"Compatibilite incorrecte dans {mixin_path}")

    native_modules = []
    for module in legacy_mixins.get("module", []):
        native_modules.append(
            {
                "id": module["id"],
                "file": module["file"],
                "name": module["name"],
                "source": "src/glue.rs",
            }
        )

    native_mixins = []
    for injection in legacy_mixins.get("injection", []):
        hook = injection.get("hook")
        target = HOOK_TARGETS.get(hook)
        if target is None:
            raise SystemExit(f"Hook Horizon non mappe : {hook}")
        row = {
            key: value
            for key, value in injection.items()
            if key not in {"hook", "hook_version"}
        }
        helper = PurePosixPath(str(row["helper"])).name
        row["helper"] = f"src/mixins/{helper}"
        row["target"] = (
            "horizon_client::game::native_hooks::Game<'_>::" + target
        )
        row.setdefault("priority", legacy_mixins.get("mod", {}).get("priority", 100))
        native_mixins.append(row)

    if not native_modules or len(native_mixins) != len(legacy_mixins.get("injection", [])):
        raise SystemExit("Declarations de modules ou d'injections natives incompletes")

    lines = [
        'id = "civ"',
        'name = "Civilisations"',
        'version = "0.1.0"',
        'description = "Simulation geopolitique fictive pour Horizon 26.3."',
        "authors = []",
        'api = "0.2"',
        'side = "both"',
        'capabilities = ["registries"]',
        "",
        "[native]",
        f'game = "{GAME}"',
        f'horizon = "{HORIZON}"',
        f'mixins = "{MIXINS}"',
    ]
    for module in native_modules:
        lines.extend(["", "[[native.module]]", *table_lines(module)])
    for mixin in native_mixins:
        lines.extend(["", "[[native.mixin]]", *table_lines(mixin)])

    manifest = "\n".join(lines) + "\n"
    parsed = tomllib.loads(manifest)
    if (
        parsed.get("id") != MOD_ID
        or parsed.get("capabilities") != ["registries"]
        or parsed.get("native", {}).get("horizon") != HORIZON
        or len(parsed.get("native", {}).get("mixin", [])) != len(native_mixins)
    ):
        raise SystemExit("Le manifeste natif genere ne passe pas sa verification")
    return manifest, legacy_mixins


def add_file(files: dict[str, bytes], path: str, data: bytes) -> None:
    normalized = PurePosixPath(path)
    if (
        normalized.is_absolute()
        or "\\" in path
        or any(part in {"", ".", ".."} for part in normalized.parts)
        or path in files
    ):
        raise SystemExit(f"Chemin de package invalide ou duplique : {path}")
    if normalized.parts[0] in {"assets", "data"} and (
        len(normalized.parts) < 3
        or any(
            part in {"", ".", ".."}
            or any(char not in CONTENT_SEGMENT_CHARS for char in part)
            for part in normalized.parts[1:]
        )
    ):
        raise SystemExit(f"Chemin de contenu invalide pour Horizon : {path}")
    files[path] = data


def build_files() -> dict[str, bytes]:
    manifest, mixin_data = source_manifest()
    files: dict[str, bytes] = {}
    add_file(files, "horizon-mod.toml", manifest.encode("utf-8"))
    add_file(
        files,
        "Cargo.toml",
        (
            '[package]\n'
            'name = "civ-horizon"\n'
            'version = "0.1.0"\n'
            'edition = "2024"\n'
            'license = "MPL-2.0"\n'
            '\n'
            '[workspace]\n'
            '\n'
            '[dependencies]\n'
            'serde = { version = "1.0", features = ["derive"] }\n'
            'serde_json = "1.0"\n'
            'sha2 = "0.10"\n'
            'thiserror = "2.0"\n'
            'uuid = { version = "1", features = ["v4"] }\n'
        ).encode("utf-8"),
    )

    horizon_lib = ADAPTER_SRC / "lib.rs"
    if not horizon_lib.is_file():
        raise SystemExit(f"Source de l'adaptateur manquante : {horizon_lib}")
    lib_bytes = horizon_lib.read_bytes()
    old_core_path = b"../../civ-core/src/"
    new_core_path = b"civ-core/src/"
    if old_core_path not in lib_bytes:
        raise SystemExit("Chemin civ-core attendu absent de civ-horizon/src/lib.rs")
    add_file(files, "src/lib.rs", lib_bytes.replace(old_core_path, new_core_path))

    for source_root, package_root in (
        (ADAPTER_SRC, "src"),
        (CORE_SRC, "src/civ-core/src"),
    ):
        for path in sorted(source_root.rglob("*.rs")):
            if path.name == "lib.rs" and source_root == ADAPTER_SRC:
                continue
            relative = path.relative_to(source_root).as_posix()
            add_file(files, f"{package_root}/{relative}", path.read_bytes())

    glue_path = MIXIN_DIR / "glue.rs"
    if not glue_path.is_file():
        raise SystemExit(f"Module Horizon manquant : {glue_path}")
    add_file(files, "src/glue.rs", glue_path.read_bytes())

    for injection in mixin_data.get("injection", []):
        helper = PurePosixPath(str(injection["helper"])).name
        source = MIXIN_DIR / helper
        if not source.is_file():
            raise SystemExit(f"Helper natif manquant : {source}")
        add_file(files, f"src/mixins/{helper}", source.read_bytes())

    scenario = NATIVE_DIR / "data" / "scenario.json"
    if not scenario.is_file():
        raise SystemExit(f"Scénario natif manquant : {scenario}")
    add_file(files, "data/civ/scenario.json", scenario.read_bytes())

    for root, prefix in ((ROOT / "data" / "civ", "data/civ"), (ROOT / "assets" / "civ", "assets/civ")):
        if root.exists():
            for path in sorted(root.rglob("*")):
                if path.is_file():
                    # Documentation is not a game asset, and its uppercase name is not a valid resource path.
                    if path.name.lower() == "readme.md":
                        continue
                    relative = path.relative_to(root).as_posix()
                    add_file(files, f"{prefix}/{relative}", path.read_bytes())

    lock_path = PROJECT_DIR / "Cargo.lock"
    if lock_path.is_file():
        add_file(files, "Cargo.lock", lock_path.read_bytes())

    return files


def write_project(files: dict[str, bytes]) -> None:
    PROJECT_DIR.mkdir(parents=True, exist_ok=True)
    for relative, data in files.items():
        if relative == "Cargo.lock":
            continue
        destination = PROJECT_DIR.joinpath(*PurePosixPath(relative).parts)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
    lock = PROJECT_DIR / "Cargo.lock"
    if lock.is_file():
        lock_data = lock.read_bytes()
        files["Cargo.lock"] = lock_data


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--out",
        type=Path,
        default=ROOT / "native" / f"{MOD_ID}.hmod",
        help="Chemin du bundle natif .hmod produit",
    )
    args = parser.parse_args()

    files = build_files()
    write_project(files)
    out = args.out.resolve()
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for relative in sorted(files):
            info = zipfile.ZipInfo(relative, STAMP)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, files[relative])

    with zipfile.ZipFile(out) as archive:
        names = archive.namelist()
        required = {
            "horizon-mod.toml",
            "Cargo.toml",
            "src/lib.rs",
            "src/glue.rs",
            "src/civ-core/src/data/mod.rs",
            "data/civ/scenario.json",
        }
        if not required.issubset(names):
            raise SystemExit(f"Fichiers requis manquants : {sorted(required - set(names))}")
        if "horizon-native.json" in names or any(
            name.startswith(("mixins/", "native/", "rust/"))
            for name in names
        ):
            raise SystemExit("Le package contient des chemins de l'ancien format")
        if any("/tests/" in name or "/target/" in name for name in names):
            raise SystemExit("Le package contient un repertoire de tests ou de build")

    digest = hashlib.sha256(out.read_bytes()).hexdigest()
    print(f"Projet source : {PROJECT_DIR}")
    print(f"Bundle cree   : {out}")
    print(f"SHA-256       : {digest}")
    print(f"Contenu       : {len(files)} fichiers, {out.stat().st_size} octets")


if __name__ == "__main__":
    main()
