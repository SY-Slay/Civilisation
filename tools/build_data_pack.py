#!/usr/bin/env python3
"""Générateur et constructeur du data-pack de production Civilisations.
Scénario : 'Arc des Frontières' avec 4 nations et 16 provinces irrégulières.
"""

from __future__ import annotations

import json
import math
import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA_DIR = ROOT / "data" / "civ"
ASSETS_DIR = ROOT / "assets" / "civ"


def create_png(width: int, height: int, rgb: tuple[int, int, int]) -> bytes:
    raw_data = bytearray()
    for y in range(height):
        raw_data.append(0)
        for x in range(width):
            r = min(255, max(0, rgb[0] + (x % 16) - 8))
            g = min(255, max(0, rgb[1] + (y % 16) - 8))
            b = min(255, max(0, rgb[2]))
            raw_data.extend([r, g, b])

    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    header = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)
    idat = zlib.compress(bytes(raw_data), level=9)
    return header + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b"")


def build_geometry():
    """Construit un maillage 4x4 de 16 provinces aux frontières irrégulières mais topologiquement exactes."""
    j_coords = {
        (0, 0): [0.0, 0.0],
        (0, 1): [260.0, 0.0],
        (0, 2): [510.0, 0.0],
        (0, 3): [770.0, 0.0],
        (0, 4): [1024.0, 0.0],
        (1, 0): [0.0, 260.0],
        (1, 1): [232.0, 280.0],
        (1, 2): [532.0, 230.0],
        (1, 3): [746.0, 286.0],
        (1, 4): [1024.0, 250.0],
        (2, 0): [0.0, 510.0],
        (2, 1): [290.0, 488.0],
        (2, 2): [496.0, 540.0],
        (2, 3): [794.0, 494.0],
        (2, 4): [1024.0, 520.0],
        (3, 0): [0.0, 770.0],
        (3, 1): [238.0, 800.0],
        (3, 2): [542.0, 742.0],
        (3, 3): [744.0, 786.0],
        (3, 4): [1024.0, 760.0],
        (4, 0): [0.0, 1024.0],
        (4, 1): [250.0, 1024.0],
        (4, 2): [520.0, 1024.0],
        (4, 3): [760.0, 1024.0],
        (4, 4): [1024.0, 1024.0],
    }

    def wavy_segment(p1, p2, offset_sign=1.0):
        dx = p2[0] - p1[0]
        dz = p2[1] - p1[1]
        dist = math.hypot(dx, dz)
        if dist < 1e-4:
            return [p1, p2]
        nx = -dz / dist
        nz = dx / dist
        m1 = [round(p1[0] + dx * 0.28 + nx * 14.0 * offset_sign, 1), round(p1[1] + dz * 0.28 + nz * 14.0 * offset_sign, 1)]
        m2 = [round(p1[0] + dx * 0.52 - nx * 12.0 * offset_sign, 1), round(p1[1] + dz * 0.52 - nz * 12.0 * offset_sign, 1)]
        m3 = [round(p1[0] + dx * 0.78 + nx * 10.0 * offset_sign, 1), round(p1[1] + dz * 0.78 + nz * 10.0 * offset_sign, 1)]
        return [p1, m1, m2, m3, p2]

    grid_provinces = {
        (0, 0): "prov_ardenne_nord",
        (0, 1): "prov_ardenne_est",
        (0, 2): "prov_kesh_marche",
        (0, 3): "prov_kesh_steppe",
        (1, 0): "prov_ardenne_val",
        (1, 1): "prov_ardenne_sud",
        (1, 2): "prov_kesh_citadelle",
        (1, 3): "prov_kesh_collines",
        (2, 0): "prov_varenne_marais",
        (2, 1): "prov_varenne_plaine",
        (2, 2): "prov_orlune_temple",
        (2, 3): "prov_orlune_pics",
        (3, 0): "prov_varenne_port",
        (3, 1): "prov_varenne_rive",
        (3, 2): "prov_orlune_vallee",
        (3, 3): "prov_orlune_sud",
    }

    h_borders = {}
    for r in range(3):
        for c in range(4):
            pa = grid_provinces[(r, c)]
            pb = grid_provinces[(r + 1, c)]
            a, b = (pa, pb) if pa < pb else (pb, pa)
            border_id = f"border_h_{r}_{r+1}_c{c}"
            p_start = j_coords[(r + 1, c)]
            p_end = j_coords[(r + 1, c + 1)]
            sign = 1.0 if (r + c) % 2 == 0 else -1.0
            pts = wavy_segment(p_start, p_end, offset_sign=sign)
            h_borders[(r, c)] = {
                "id": border_id,
                "a": a,
                "b": b,
                "kind": "river" if r == 1 else "land",
                "points": pts,
                "natural_barrier": (r == 1),
                "crossing_cost": 2 if r == 1 else 1,
            }

    v_borders = {}
    for r in range(4):
        for c in range(3):
            pa = grid_provinces[(r, c)]
            pb = grid_provinces[(r, c + 1)]
            a, b = (pa, pb) if pa < pb else (pb, pa)
            border_id = f"border_v_r{r}_c{c}_{c+1}"
            p_start = j_coords[(r, c + 1)]
            p_end = j_coords[(r + 1, c + 1)]
            sign = -1.0 if (r + c) % 2 == 0 else 1.0
            pts = wavy_segment(p_start, p_end, offset_sign=sign)
            v_borders[(r, c)] = {
                "id": border_id,
                "a": a,
                "b": b,
                "kind": "mountain" if c == 1 else "land",
                "points": pts,
                "natural_barrier": (c == 1),
                "crossing_cost": 2 if c == 1 else 1,
            }

    all_borders = []
    for border in h_borders.values():
        all_borders.append(border)
    for border in v_borders.values():
        all_borders.append(border)

    all_borders.sort(key=lambda b: b["id"])

    provinces_geom = {}
    for r in range(4):
        for c in range(4):
            pid = grid_provinces[(r, c)]

            if c > 0:
                w_pts = [p for p in v_borders[(r, c - 1)]["points"]]
            else:
                p1 = j_coords[(r, c)]
                p2 = j_coords[(r + 1, c)]
                w_pts = [p1, [p1[0], round((p1[1] + p2[1]) * 0.5, 1)], p2]

            if r < 3:
                s_pts = [p for p in h_borders[(r, c)]["points"]]
            else:
                p1 = j_coords[(r + 1, c)]
                p2 = j_coords[(r + 1, c + 1)]
                s_pts = [p1, [round((p1[0] + p2[0]) * 0.5, 1), p1[1]], p2]

            if c < 3:
                e_pts = [p for p in reversed(v_borders[(r, c)]["points"])]
            else:
                p1 = j_coords[(r + 1, c + 1)]
                p2 = j_coords[(r, c + 1)]
                e_pts = [p1, [p1[0], round((p1[1] + p2[1]) * 0.5, 1)], p2]

            if r > 0:
                n_pts = [p for p in reversed(h_borders[(r - 1, c)]["points"])]
            else:
                p1 = j_coords[(r, c + 1)]
                p2 = j_coords[(r, c)]
                n_pts = [p1, [round((p1[0] + p2[0]) * 0.5, 1), p1[1]], p2]

            polygon_pts = []
            polygon_pts.extend(w_pts[:-1])
            polygon_pts.extend(s_pts[:-1])
            polygon_pts.extend(e_pts[:-1])
            polygon_pts.extend(n_pts[:-1])

            neighbors = []
            border_ids = []

            if r > 0:
                neighbors.append(grid_provinces[(r - 1, c)])
                border_ids.append(h_borders[(r - 1, c)]["id"])
            if r < 3:
                neighbors.append(grid_provinces[(r + 1, c)])
                border_ids.append(h_borders[(r, c)]["id"])
            if c > 0:
                neighbors.append(grid_provinces[(r, c - 1)])
                border_ids.append(v_borders[(r, c - 1)]["id"])
            if c < 3:
                neighbors.append(grid_provinces[(r, c + 1)])
                border_ids.append(v_borders[(r, c)]["id"])

            neighbors.sort()
            border_ids.sort()

            cx = sum(p[0] for p in polygon_pts) / len(polygon_pts)
            cz = sum(p[1] for p in polygon_pts) / len(polygon_pts)

            area = 0.0
            n = len(polygon_pts)
            for i in range(n):
                cur = polygon_pts[i]
                nxt = polygon_pts[(i + 1) % n]
                area += (cur[0] * nxt[1] - cur[1] * nxt[0])
            area = abs(area * 0.5)

            provinces_geom[pid] = {
                "polygon": polygon_pts,
                "center": [round(cx, 2), round(cz, 2)],
                "area_blocks": int(round(area)),
                "neighbors": neighbors,
                "border_ids": border_ids,
            }

    return all_borders, provinces_geom


def generate_all():
    print("Génération du pack de données sous data/civ/ ...")

    # 1. Création des répertoires
    subdirs = [
        "cultures",
        "religions",
        "ideologies",
        "governments",
        "resources",
        "technologies",
        "units",
        "missiles",
        "events",
        "names",
        "localization",
        "nations",
        "provinces",
        "scenarios",
    ]
    for sub in subdirs:
        (DATA_DIR / sub).mkdir(parents=True, exist_ok=True)

    # 2. Cultures
    cultures = [
        {
            "schema_version": 1,
            "id": "ardennic",
            "name": "Ardennique",
            "adjective": "ardennois",
            "description": "Culture démocratique des vallées du nord, attachée à l'État de droit et aux assemblées communales.",
            "language": "fr_FR",
            "family": "western_valley",
            "values": ["stabilité", "délibération", "solidarité"],
            "traditions": ["conseil des communes", "foire aux moissons"],
            "name_pool": "names_ardennic",
            "color": "#3A7BD5",
            "modifiers": [],
            "tags": ["democratic", "valley"],
        },
        {
            "schema_version": 1,
            "id": "keshi",
            "name": "Keshi",
            "adjective": "keshi",
            "description": "Culture martiale des steppes et forteresses de l'est, valorisant la discipline militaire et l'honneur du serment.",
            "language": "fr_FR",
            "family": "eastern_steppe",
            "values": ["discipline", "force", "devoir"],
            "traditions": ["serment d'armes", "tournoi d'acier"],
            "name_pool": "names_keshi",
            "color": "#E8590C",
            "modifiers": [],
            "tags": ["militarist", "steppe"],
        },
        {
            "schema_version": 1,
            "id": "varennois",
            "name": "Varennois",
            "adjective": "varennois",
            "description": "Culture maritime et commerçante des estuaires du sud-ouest, axée sur les comptoirs et la diplomatie contractuelle.",
            "language": "fr_FR",
            "family": "coastal_league",
            "values": ["commerce", "pragmatisme", "prospérité"],
            "traditions": ["foire maritime", "charte des guildes"],
            "name_pool": "names_varennois",
            "color": "#12B886",
            "modifiers": [],
            "tags": ["mercantile", "coastal"],
        },
        {
            "schema_version": 1,
            "id": "orlunian",
            "name": "Orlunien",
            "adjective": "orlunien",
            "description": "Culture contemplative et érudite des hautes vallées du sud-est, guidée par les rites astraux et la piété.",
            "language": "fr_FR",
            "family": "highland_monastic",
            "values": ["foi", "connaissance", "harmonie"],
            "traditions": ["veille des astres", "jeûne du solstice"],
            "name_pool": "names_orlunian",
            "color": "#9C36B5",
            "modifiers": [],
            "tags": ["theocratic", "mountain"],
        },
    ]
    for c in cultures:
        (DATA_DIR / "cultures" / f"{c['id']}.json").write_text(json.dumps(c, indent=2, ensure_ascii=False), encoding="utf-8")

    # 3. Religions
    religions = [
        {
            "schema_version": 1,
            "id": "rationalism",
            "name": "Rationalisme civique",
            "description": "Philosophie morale centrée sur la justice constitutionnelle et le bien public.",
            "family": "civic_creed",
            "tenets": ["primauté de la loi", "bien commun", "recherche de vérité"],
            "tolerance": 75.0,
            "holy_sites": ["prov_ardenne_val"],
            "clergy_title": "Gardien de la Charte",
            "modifiers": [],
            "tags": ["rationalist", "civic"],
        },
        {
            "schema_version": 1,
            "id": "martial_order",
            "name": "Ordre du Serment d'Acier",
            "description": "Culte martial et traditionnel honorant les ancêtres guerriers et le devoir sans faille.",
            "family": "warrior_creed",
            "tenets": ["fidélité au chef", "gloire des armes", "sacrifice pour le clan"],
            "tolerance": 40.0,
            "holy_sites": ["prov_kesh_citadelle"],
            "clergy_title": "Commandeur des Rites",
            "modifiers": [],
            "tags": ["militarist", "ancestral"],
        },
        {
            "schema_version": 1,
            "id": "pragmatism",
            "name": "Communion des Flots",
            "description": "Croyance côtière pragmatique célébrant les vents favorables et l'harmonie des contrats.",
            "family": "maritime_creed",
            "tenets": ["fluidité des échanges", "respect absolu des pactes", "prosperité partagée"],
            "tolerance": 85.0,
            "holy_sites": ["prov_varenne_port"],
            "clergy_title": "Maître de la Marée",
            "modifiers": [],
            "tags": ["mercantile", "coastal"],
        },
        {
            "schema_version": 1,
            "id": "mysticism",
            "name": "Culte des Astres Éternels",
            "description": "Religion mystique et théocratique guidée par les constellations et l'ascèse.",
            "family": "astral_creed",
            "tenets": ["alignement céleste", "pureté spirituelle", "soumission au destin"],
            "tolerance": 50.0,
            "holy_sites": ["prov_orlune_temple"],
            "clergy_title": "Archonte du Voile",
            "modifiers": [],
            "tags": ["theocratic", "astral"],
        },
    ]
    for r in religions:
        (DATA_DIR / "religions" / f"{r['id']}.json").write_text(json.dumps(r, indent=2, ensure_ascii=False), encoding="utf-8")

    # 4. Idéologies
    ideologies = [
        {
            "schema_version": 1,
            "id": "civic_republicanism",
            "name": "Républicanisme civique",
            "description": "Doctrine fondée sur la démocratie représentative, l'État de droit et l'égalité citoyenne.",
            "family": "liberal",
            "values": ["liberté politique", "délibération", "pluralisme"],
            "government_preferences": ["republic", "democracy"],
            "modifiers": [],
            "tags": ["democratic"],
        },
        {
            "schema_version": 1,
            "id": "militarist_autocracy",
            "name": "Autocratie martiale",
            "description": "Doctrine de centralisation autoritaire au service de la puissance militaire et de l'ordre intérieur.",
            "family": "militarist",
            "values": ["ordre hiérarchique", "puissance nationale", "discipline"],
            "government_preferences": ["autocracy", "junta"],
            "modifiers": [],
            "tags": ["authoritarian"],
        },
        {
            "schema_version": 1,
            "id": "commercial_oligarchy",
            "name": "Oligarchie marchande",
            "description": "Modèle politique axé sur la prospérité financière, l'influence des guildes et le libre-échange.",
            "family": "mercantile",
            "values": ["libre entreprise", "souveraineté financière", "pragmatisme"],
            "government_preferences": ["oligarchy", "republic"],
            "modifiers": [],
            "tags": ["plutocratic"],
        },
        {
            "schema_version": 1,
            "id": "devotional_theocracy",
            "name": "Théocratie dévotionnelle",
            "description": "Organisation sacrée où le gouvernement applique les décrets divins et préserve l'orthodoxie.",
            "family": "theocratic",
            "values": ["soumission sacrée", "justice divine", "dévotion"],
            "government_preferences": ["theocracy", "council"],
            "modifiers": [],
            "tags": ["religious"],
        },
    ]
    for i in ideologies:
        (DATA_DIR / "ideologies" / f"{i['id']}.json").write_text(json.dumps(i, indent=2, ensure_ascii=False), encoding="utf-8")

    # 5. Gouvernements
    governments = [
        {
            "schema_version": 1,
            "id": "republic",
            "name": "République constitutionnelle",
            "description": "Régime représentatif doté d'une assemblée élue et de contre-pouvoirs garantis.",
            "category": "republic",
            "legitimacy_source": "elections",
            "succession": "election",
            "term_length_days": 1460,
            "modifiers": [],
            "tags": ["constitutional"],
        },
        {
            "schema_version": 1,
            "id": "autocracy",
            "name": "Commandement suprême autocratique",
            "description": "Régime autoritaire centralisé dirigé par un chef suprême incontesté.",
            "category": "autocracy",
            "legitimacy_source": "military",
            "succession": "appointment",
            "term_length_days": 0,
            "modifiers": [],
            "tags": ["militarist"],
        },
        {
            "schema_version": 1,
            "id": "oligarchy",
            "name": "Grand Conseil des marchands",
            "description": "Gouvernement collégial composé des syndics des grandes corporations et maisons de négoce.",
            "category": "oligarchy",
            "legitimacy_source": "competence",
            "succession": "council_vote",
            "term_length_days": 1825,
            "modifiers": [],
            "tags": ["patrician"],
        },
        {
            "schema_version": 1,
            "id": "theocracy",
            "name": "Haut Conclave sacerdotal",
            "description": "Gouvernement des dignitaires religieux veillant sur l'orthodoxie des lois et des mœurs.",
            "category": "theocracy",
            "legitimacy_source": "religion",
            "succession": "council_vote",
            "term_length_days": 0,
            "modifiers": [],
            "tags": ["sacred"],
        },
    ]
    for g in governments:
        (DATA_DIR / "governments" / f"{g['id']}.json").write_text(json.dumps(g, indent=2, ensure_ascii=False), encoding="utf-8")

    # 6. Ressources
    resources = [
        {
            "schema_version": 1,
            "id": "iron_ore",
            "name": "Minerai de fer",
            "description": "Matière première indispensable pour l'industrie lourde, les rails et l'armement.",
            "category": "metal",
            "unit": "ton",
            "base_value": 120.0,
            "strategic": True,
            "modifiers": [],
            "tags": ["industrial", "strategic"],
        },
        {
            "schema_version": 1,
            "id": "coal",
            "name": "Charbon minier",
            "description": "Combustible fossile essentiel à la sidérurgie et à l'alimentation des centrales thermiques.",
            "category": "energy",
            "unit": "ton",
            "base_value": 80.0,
            "strategic": True,
            "modifiers": [],
            "tags": ["energy", "bulk"],
        },
        {
            "schema_version": 1,
            "id": "grain",
            "name": "Céréales de plaine",
            "description": "Production agricole assurant la subsistance des populations urbaines et la stabilité.",
            "category": "food",
            "unit": "bushel",
            "base_value": 30.0,
            "strategic": False,
            "modifiers": [],
            "tags": ["agriculture"],
        },
        {
            "schema_version": 1,
            "id": "oil",
            "name": "Hydrocarbures bruts",
            "description": "Ressource stratégique vitale pour les unités mécanisées, l'aviation et l'industrie chimique.",
            "category": "energy",
            "unit": "barrel",
            "base_value": 150.0,
            "strategic": True,
            "modifiers": [],
            "tags": ["hydrocarbon", "strategic"],
        },
        {
            "schema_version": 1,
            "id": "rare_earths",
            "name": "Minéraux rares",
            "description": "Éléments indispensables à l'électronique de pointe, aux radars et au guidage balistique.",
            "category": "rare",
            "unit": "unit",
            "base_value": 250.0,
            "strategic": True,
            "modifiers": [],
            "tags": ["hightech", "strategic"],
        },
    ]
    for res in resources:
        (DATA_DIR / "resources" / f"{res['id']}.json").write_text(json.dumps(res, indent=2, ensure_ascii=False), encoding="utf-8")

    # 7. Technologies
    technologies = [
        {
            "schema_version": 1,
            "id": "tech_rail_logistics",
            "name": "Logistique ferroviaire",
            "description": "Réseau ferré moderne permettant le déplacement rapide des armées et le ravitaillement.",
            "era": 2,
            "category": "economy",
            "cost": 400.0,
            "research_time_days": 90,
            "prerequisites": [],
            "effects": [],
            "unlocks": {"units": [], "missiles": [], "buildings": [], "events": [], "modifiers": []},
            "repeatable": False,
            "max_level": 1,
            "ai_weight": 1.2,
            "tags": ["logistics"],
        },
        {
            "schema_version": 1,
            "id": "tech_steel_metallurgy",
            "name": "Métallurgie de l'acier",
            "description": "Fonderies avancées améliorant le rendement industriel et la robustesse des blindages.",
            "era": 2,
            "category": "industry",
            "cost": 500.0,
            "research_time_days": 120,
            "prerequisites": [],
            "effects": [],
            "unlocks": {"units": ["unit_line_infantry"], "missiles": [], "buildings": [], "events": [], "modifiers": []},
            "repeatable": False,
            "max_level": 1,
            "ai_weight": 1.1,
            "tags": ["industry"],
        },
        {
            "schema_version": 1,
            "id": "tech_radio_communications",
            "name": "Télécommunications radio",
            "description": "Transmissions hertziennes fiables reliant l'état-major aux divisions en campagne.",
            "era": 3,
            "category": "computing",
            "cost": 600.0,
            "research_time_days": 150,
            "prerequisites": [],
            "effects": [],
            "unlocks": {"units": [], "missiles": [], "buildings": [], "events": [], "modifiers": []},
            "repeatable": False,
            "max_level": 1,
            "ai_weight": 1.0,
            "tags": ["communications"],
        },
        {
            "schema_version": 1,
            "id": "tech_combined_arms",
            "name": "Doctrine interarmes",
            "description": "Coordination tactique étroite entre infanterie motorisée, blindés et appui d'artillerie.",
            "era": 3,
            "category": "military",
            "cost": 700.0,
            "research_time_days": 180,
            "prerequisites": ["tech_rail_logistics"],
            "effects": [],
            "unlocks": {"units": ["unit_mechanized_guard", "unit_heavy_armor"], "missiles": [], "buildings": [], "events": [], "modifiers": []},
            "repeatable": False,
            "max_level": 1,
            "ai_weight": 1.3,
            "tags": ["tactics"],
        },
        {
            "schema_version": 1,
            "id": "tech_radar_early_warning",
            "name": "Détection radar avancée",
            "description": "Stations de surveillance aérienne et côtière fournissant une alerte précoce contre les frappes.",
            "era": 3,
            "category": "intelligence",
            "cost": 850.0,
            "research_time_days": 200,
            "prerequisites": ["tech_radio_communications"],
            "effects": [],
            "unlocks": {"units": [], "missiles": [], "buildings": [], "events": [], "modifiers": []},
            "repeatable": False,
            "max_level": 1,
            "ai_weight": 0.9,
            "tags": ["radar"],
        },
        {
            "schema_version": 1,
            "id": "tech_ballistic_propulsion",
            "name": "Propulsion balistique",
            "description": "Moteurs-fusées à ergols stables ouvrant la voie aux frappes guidées de longue portée.",
            "era": 4,
            "category": "missile",
            "cost": 1000.0,
            "research_time_days": 240,
            "prerequisites": ["tech_radar_early_warning"],
            "effects": [],
            "unlocks": {"units": ["unit_missile_corps"], "missiles": ["missile_short_range", "missile_cruise"], "buildings": [], "events": [], "modifiers": []},
            "repeatable": False,
            "max_level": 1,
            "ai_weight": 1.5,
            "tags": ["strategic"],
        },
    ]
    for tech in technologies:
        (DATA_DIR / "technologies" / f"{tech['id']}.json").write_text(json.dumps(tech, indent=2, ensure_ascii=False), encoding="utf-8")

    # 8. Unités
    units = [
        {
            "schema_version": 1,
            "id": "unit_line_infantry",
            "name": "Infanterie de ligne",
            "description": "Troupe standard entraînée, disciplinée et solide dans les retranchements.",
            "category": "infantry",
            "era": 2,
            "cost": 120.0,
            "upkeep": 3.0,
            "manpower": 1000,
            "attack": 28.0,
            "defense": 45.0,
            "mobility": 35.0,
            "morale": 60.0,
            "training_time_days": 30,
            "prerequisites": [],
            "counters": [],
            "weak_against": ["unit_heavy_armor"],
            "tags": ["core"],
        },
        {
            "schema_version": 1,
            "id": "unit_mechanized_guard",
            "name": "Garde mécanisée",
            "description": "Infanterie montée sur véhicules blindés rapides, offrant une excellente mobilité tactique.",
            "category": "mechanized",
            "era": 3,
            "cost": 250.0,
            "upkeep": 8.0,
            "manpower": 600,
            "attack": 55.0,
            "defense": 50.0,
            "mobility": 65.0,
            "morale": 70.0,
            "training_time_days": 60,
            "prerequisites": ["tech_combined_arms"],
            "counters": ["unit_line_infantry"],
            "weak_against": ["unit_heavy_armor"],
            "tags": ["mobile"],
        },
        {
            "schema_version": 1,
            "id": "unit_heavy_armor",
            "name": "Bataillon blindé lourd",
            "description": "Chars de combat protégés capables de percer les lignes ennemies fortifiées.",
            "category": "armor",
            "era": 3,
            "cost": 400.0,
            "upkeep": 14.0,
            "manpower": 400,
            "attack": 75.0,
            "defense": 65.0,
            "mobility": 50.0,
            "morale": 75.0,
            "training_time_days": 90,
            "prerequisites": ["tech_combined_arms"],
            "counters": ["unit_line_infantry", "unit_mechanized_guard"],
            "weak_against": ["unit_artillery_battery"],
            "tags": ["spearhead"],
        },
        {
            "schema_version": 1,
            "id": "unit_artillery_battery",
            "name": "Batterie d'artillerie lourde",
            "description": "Pièces de gros calibre pilonnant les défenses adverses et désorganisant les concentrations.",
            "category": "artillery",
            "era": 2,
            "cost": 200.0,
            "upkeep": 6.0,
            "manpower": 300,
            "attack": 60.0,
            "defense": 25.0,
            "mobility": 25.0,
            "morale": 55.0,
            "training_time_days": 45,
            "prerequisites": ["tech_steel_metallurgy"],
            "counters": ["unit_heavy_armor"],
            "weak_against": ["unit_mechanized_guard"],
            "tags": ["fire_support"],
        },
        {
            "schema_version": 1,
            "id": "unit_missile_corps",
            "name": "Régiment de lanceurs mobiles",
            "description": "Véhicules lance-roquettes et missiles balistiques tactiques pour frappes en profondeur.",
            "category": "missile",
            "era": 4,
            "cost": 600.0,
            "upkeep": 20.0,
            "manpower": 250,
            "attack": 90.0,
            "defense": 20.0,
            "mobility": 40.0,
            "morale": 70.0,
            "training_time_days": 120,
            "prerequisites": ["tech_ballistic_propulsion"],
            "counters": [],
            "weak_against": ["unit_mechanized_guard"],
            "tags": ["strategic_asset"],
        },
    ]
    for u in units:
        (DATA_DIR / "units" / f"{u['id']}.json").write_text(json.dumps(u, indent=2, ensure_ascii=False), encoding="utf-8")

    # 9. Missiles
    missiles = [
        {
            "id": "missile_short_range",
            "name": "Missile tactique d'artillerie",
            "description": "Vecteur balistique sol-sol pour l'appui tactique au-delà de la ligne de front.",
            "kind": "ballistic",
            "range_km": 300.0,
            "speed_kmh": 2800.0,
            "flight_profile": "ballistic",
            "accuracy_m": 100.0,
            "warhead": {
                "kind": "conventional",
                "yield_kt": 0,
                "blast_radius_blocks": 24,
                "effects": [],
                "fallout": False,
                "escalation_delta": 0,
            },
            "cost": 600.0,
            "build_time_days": 20,
            "max_stock": 30,
            "defense_penetration": 0.25,
            "interception": {
                "base_chance": 0.45,
                "counter_types": [],
                "terminal_maneuver": False,
            },
            "launch_platforms": ["mobile_launcher"],
            "explosion_effects": {
                "particle": "civ:missile_impact",
                "sound": "civ:missile_impact",
                "flash_color": "#FFAA33",
                "shake_radius": 64,
                "smoke_ticks": 200,
                "orbital_laser_hook": False,
            },
            "tags": ["tactical"],
        },
        {
            "id": "missile_cruise",
            "name": "Missile de croisière rase-mottes",
            "description": "Engin guidé volant à très basse altitude pour déjouer les radars de détection.",
            "kind": "cruise",
            "range_km": 1200.0,
            "speed_kmh": 950.0,
            "flight_profile": "low",
            "accuracy_m": 15.0,
            "warhead": {
                "kind": "conventional",
                "yield_kt": 0,
                "blast_radius_blocks": 32,
                "effects": [],
                "fallout": False,
                "escalation_delta": 0,
            },
            "cost": 1200.0,
            "build_time_days": 45,
            "max_stock": 20,
            "defense_penetration": 0.60,
            "interception": {
                "base_chance": 0.35,
                "counter_types": [],
                "terminal_maneuver": True,
            },
            "launch_platforms": ["mobile_launcher", "silo"],
            "explosion_effects": {
                "particle": "civ:missile_impact",
                "sound": "civ:missile_impact",
                "flash_color": "#FF8822",
                "shake_radius": 80,
                "smoke_ticks": 240,
                "orbital_laser_hook": False,
            },
            "tags": ["precision"],
        },
        {
            "id": "missile_strategic_ballistic",
            "name": "Missile balistique stratégique lourd",
            "description": "Vecteur à haute vélocité atteignant l'espace suborbital pour frapper les centres névralgiques.",
            "kind": "ballistic",
            "range_km": 3500.0,
            "speed_kmh": 12000.0,
            "flight_profile": "ballistic",
            "accuracy_m": 50.0,
            "warhead": {
                "kind": "conventional",
                "yield_kt": 0,
                "blast_radius_blocks": 56,
                "effects": [],
                "fallout": False,
                "escalation_delta": 5,
            },
            "cost": 3500.0,
            "build_time_days": 120,
            "max_stock": 10,
            "defense_penetration": 0.80,
            "interception": {
                "base_chance": 0.25,
                "counter_types": [],
                "terminal_maneuver": True,
            },
            "launch_platforms": ["silo"],
            "explosion_effects": {
                "particle": "civ:missile_impact",
                "sound": "civ:missile_impact",
                "flash_color": "#FF4411",
                "shake_radius": 120,
                "smoke_ticks": 360,
                "orbital_laser_hook": False,
            },
            "tags": ["deterrence"],
        },
    ]
    for m in missiles:
        (DATA_DIR / "missiles" / f"{m['id']}.json").write_text(json.dumps(m, indent=2, ensure_ascii=False), encoding="utf-8")

    # 10. Événements
    events = [
        {
            "schema_version": 1,
            "id": "event_border_skirmish",
            "name": "Incident de démarcation frontalière",
            "description": "Une escouade armée a franchi les bornes de délimitation dans un secteur boisé contesté.",
            "kind": "diplomatic",
            "weight": 1.2,
            "cooldown_days": 60,
            "max_fires": 5,
            "trigger": {
                "conditions": [],
                "min_date": {"year": 2026, "month": 1, "day": 1},
                "chance": 0.8,
            },
            "choices": [
                {
                    "id": "protest_diplomatically",
                    "text": "Envoyer une note de protestation solennelle",
                    "effects": [],
                    "ai_weight": 0.7,
                },
                {
                    "id": "reinforce_border",
                    "text": "Dépêcher des renforts et fortifier la ligne",
                    "effects": [],
                    "ai_weight": 0.3,
                },
            ],
            "severity": 25.0,
            "tags": ["border", "crisis"],
        },
        {
            "schema_version": 1,
            "id": "event_resource_boom",
            "name": "Découverte d'un gisement riche",
            "description": "Des géomètres ont cartographié une formation minérale d'une abondance remarquable.",
            "kind": "economic",
            "weight": 1.0,
            "cooldown_days": 120,
            "max_fires": 3,
            "trigger": {
                "conditions": [],
                "min_date": {"year": 2026, "month": 3, "day": 1},
                "chance": 0.5,
            },
            "choices": [
                {
                    "id": "national_exploitation",
                    "text": "Créer une régie publique pour valoriser la ressource",
                    "effects": [],
                    "ai_weight": 0.6,
                },
                {
                    "id": "concession_auctions",
                    "text": "Mettre aux enchères des concessions de forage",
                    "effects": [],
                    "ai_weight": 0.4,
                },
            ],
            "severity": 10.0,
            "tags": ["economy", "mining"],
        },
        {
            "schema_version": 1,
            "id": "event_trade_delegation",
            "name": "Ambassade des marchands de Varenne",
            "description": "Une délégation commerciale propose un accord préférentiel sur les droits de douane maritimes.",
            "kind": "diplomatic",
            "weight": 1.5,
            "cooldown_days": 90,
            "max_fires": 6,
            "trigger": {
                "conditions": [],
                "min_date": {"year": 2026, "month": 2, "day": 1},
                "chance": 0.7,
            },
            "choices": [
                {
                    "id": "sign_accord",
                    "text": "Signer les accords d'ouverture tarifaire",
                    "effects": [],
                    "ai_weight": 0.8,
                },
                {
                    "id": "refuse_protectionism",
                    "text": "Maintenir les barrières pour protéger la production locale",
                    "effects": [],
                    "ai_weight": 0.2,
                },
            ],
            "severity": 15.0,
            "tags": ["trade", "diplomacy"],
        },
        {
            "schema_version": 1,
            "id": "event_religious_schism",
            "name": "Dispute doctrinale au sanctuaire",
            "description": "Des prédicateurs itinérants remettent en cause l'autorité sacerdotale établie.",
            "kind": "political",
            "weight": 0.8,
            "cooldown_days": 180,
            "max_fires": 2,
            "trigger": {
                "conditions": [],
                "min_date": {"year": 2026, "month": 6, "day": 1},
                "chance": 0.4,
            },
            "choices": [
                {
                    "id": "conciliate",
                    "text": "Organiser un synode théologique de conciliation",
                    "effects": [],
                    "ai_weight": 0.5,
                },
                {
                    "id": "ban_heresy",
                    "text": "Interdire les prédications dissidentes",
                    "effects": [],
                    "ai_weight": 0.5,
                },
            ],
            "severity": 30.0,
            "tags": ["religion", "unrest"],
        },
        {
            "schema_version": 1,
            "id": "event_military_parade",
            "name": "Grande parade de la Citadelle",
            "description": "L'armée étale sa puissance mécanique et ses pièces balistiques sous les acclamations populaires.",
            "kind": "military",
            "weight": 1.0,
            "cooldown_days": 120,
            "max_fires": 4,
            "trigger": {
                "conditions": [],
                "min_date": {"year": 2026, "month": 4, "day": 1},
                "chance": 0.6,
            },
            "choices": [
                {
                    "id": "national_pride",
                    "text": "Saluer la détermination héroïque de la nation",
                    "effects": [],
                    "ai_weight": 0.9,
                },
                {
                    "id": "austere_focus",
                    "text": "Rediriger les fonds du défilé vers l'entraînement",
                    "effects": [],
                    "ai_weight": 0.1,
                },
            ],
            "severity": 10.0,
            "tags": ["military", "prestige"],
        },
    ]
    for ev in events:
        (DATA_DIR / "events" / f"{ev['id']}.json").write_text(json.dumps(ev, indent=2, ensure_ascii=False), encoding="utf-8")

    # 11. Réservoirs de noms
    names_pools = [
        {
            "schema_version": 1,
            "id": "names_ardennic",
            "culture": "ardennic",
            "language": "fr_FR",
            "nation_names": ["Ardenne", "Valdorie", "Montclaire", "Rive-Haute"],
            "nation_adjectives": ["ardennois", "valdorien", "montclairien", "rivois"],
            "city_names": ["Mireval", "Hautpont", "Belvédère", "Brumeval", "Clairefont", "Montagne-Bleue"],
            "region_names": ["Bassin de la Brume", "Haute-Ardenne", "Plateau des Vergers"],
            "leader_names": {
                "given": ["Mira", "Tomas", "Nael", "Élise", "Julien", "Clara"],
                "family": ["Valen", "Ors", "Drev", "Sorel", "Castel", "Mercier"],
                "titles": ["Présidente", "Président", "Consul", "Chancelier"],
                "epithets": ["le Prudent", "la Juste", "le Bâtisseur", "la Ferme"],
            },
            "army_names": ["1re Armée du Val", "Garde Nationale", "Brigade de l'Aube"],
            "operation_names": ["Aube Claire", "Bastion Serein", "Vigilance"],
        },
        {
            "schema_version": 1,
            "id": "names_keshi",
            "culture": "keshi",
            "language": "fr_FR",
            "nation_names": ["Kesh", "Karkovie", "Fer-Gris", "Vorn"],
            "nation_adjectives": ["keshi", "karkovien", "vornien", "ferriste"],
            "city_names": ["Kesh-la-Haute", "Fort-Enclume", "Borne-de-Fer", "Roc-d'Acier", "Vanguard"],
            "region_names": ["Steppe Noire", "Crête du Faucon", "Détroit Sanglant"],
            "leader_names": {
                "given": ["Varek", "Goran", "Kael", "Dara", "Zarek", "Boris"],
                "family": ["Thorne", "Malek", "Karkov", "Rostov", "Steele", "Volf"],
                "titles": ["Généralissime", "Commandeur", "Maréchal", "Grand Chambellan"],
                "epithets": ["l'Inflexible", "le Victorieux", "le Terrible", "le Poing"],
            },
            "army_names": ["1er Corps de Rupture", "Garde d'Acier", "Division Cuirassée Karkov"],
            "operation_names": ["Tonnerre Noir", "Frappe Déterminée", "Enclume d'Or"],
        },
        {
            "schema_version": 1,
            "id": "names_varennois",
            "culture": "varennois",
            "language": "fr_FR",
            "nation_names": ["Varenne", "Ligue d'Argent", "Riviera", "Comptoirs Unis"],
            "nation_adjectives": ["varennois", "argenté", "rivierain", "ligueur"],
            "city_names": ["Varenne-Maritime", "Port-Franc", "Quai-des-Flots", "Lagune-Verte", "Comptoir-Doré"],
            "region_names": ["Grand Delta", "Côte d'Émeraude", "Mares Salines"],
            "leader_names": {
                "given": ["Alvise", "Matteo", "Diane", "Helena", "Soren", "Lucia"],
                "family": ["Contarini", "Marin", "Dolfin", "Badoer", "Morosini", "Pisani"],
                "titles": ["Doge", "Grand Prieur", "Syndic", "Gouverneur"],
                "epithets": ["l'Avisé", "le Pacifique", "le Négociateur", "l'Audacieux"],
            },
            "army_names": ["Garde Municipale des Guildes", "Légion des Corsaires", "Régiment de Marine"],
            "operation_names": ["Grand Vent", "Balance d'Or", "Haute Marée"],
        },
        {
            "schema_version": 1,
            "id": "names_orlunian",
            "culture": "orlunian",
            "language": "fr_FR",
            "nation_names": ["Orlune", "Sanctuaire Céleste", "Thuléa", "Astraea"],
            "nation_adjectives": ["orlunien", "céleste", "thuléen", "astral"],
            "city_names": ["Orlune-Sanctuaire", "Haut-Dôme", "Pic-des-Astres", "Temple-Solitaire", "Étoile-du-Nord"],
            "region_names": ["Vallée Lumineuse", "Massif Sacré", "Gorges du Destin"],
            "leader_names": {
                "given": ["Elysia", "Theron", "Selen", "Cassian", "Lyra", "Vael"],
                "family": ["Solas", "Astris", "Aethel", "Vane", "Kaelen", "Morr"],
                "titles": ["Archonte", "Hiérophante", "Grand Prélat", "Gardienne"],
                "epithets": ["l'Éclairé", "l'Immaculée", "la Prophétesse", "le Sage"],
            },
            "army_names": ["Croisés du Voile", "Garde Temple", "Sentinelles d'Orlune"],
            "operation_names": ["Rayonnement", "Aube Stellaire", "Purification"],
        },
    ]
    for n in names_pools:
        (DATA_DIR / "names" / f"{n['id']}.json").write_text(json.dumps(n, indent=2, ensure_ascii=False), encoding="utf-8")

    # 12. Localisation
    loc_fr = {
        "schema_version": 1,
        "language": "fr_FR",
        "fallback_language": "en_US",
        "entries": {
            "civ.game.name": "Civilisations",
            "civ.scenario.arc_des_frontieres": "Arc des Frontières",
            "civ.scenario.desc": "Quatre puissances rivales se disputent l'hégémonie dans une région fracturée par les fleuves et les reliefs.",
            "civ.nation.ardenne": "République d'Ardenne",
            "civ.nation.kesh": "Autocratie de Kesh",
            "civ.nation.varenne": "Ligue marchande de Varenne",
            "civ.nation.orlune": "Sanctuaire d'Orlune",
            "civ.command.status": "Statut de la simulation : tick {tick}, date {date}",
            "civ.command.ping": "Pong déterministe !",
        },
    }
    loc_en = {
        "schema_version": 1,
        "language": "en_US",
        "fallback_language": "fr_FR",
        "entries": {
            "civ.game.name": "Civilizations",
            "civ.scenario.arc_des_frontieres": "Borderlands Arc",
            "civ.scenario.desc": "Four rival powers compete for hegemony in a fractured geopolitical landscape.",
            "civ.nation.ardenne": "Republic of Ardenne",
            "civ.nation.kesh": "Autocracy of Kesh",
            "civ.nation.varenne": "Merchant League of Varenne",
            "civ.nation.orlune": "Sanctuary of Orlune",
            "civ.command.status": "Simulation status: tick {tick}, date {date}",
            "civ.command.ping": "Deterministic pong!",
        },
    }
    (DATA_DIR / "localization" / "fr_fr.json").write_text(json.dumps(loc_fr, indent=2, ensure_ascii=False), encoding="utf-8")
    (DATA_DIR / "localization" / "en_us.json").write_text(json.dumps(loc_en, indent=2, ensure_ascii=False), encoding="utf-8")

    # 13. Géométrie et Provinces
    borders, provinces_geom = build_geometry()

    # Propriétés thématiques pour chaque province
    province_specs = {
        # ARDENNE (Nord-Ouest)
        "prov_ardenne_nord": {
            "name": "Côte d'Ardenne",
            "culture": "ardennic",
            "religion": "rationalism",
            "terrain": "coast",
            "climate": {"temperature_c": 14.5, "rainfall_mm": 850.0, "seasonality": 0.45},
            "pop_total": 180000, "pop_urban": 70000, "pop_rural": 110000,
            "resources": [{"resource": "grain", "abundance": 75.0, "reserve": 200000.0}],
            "owner": "ardenne", "controller": "ardenne", "capital_of": None,
            "fortification": 1, "unrest": 5.0, "supply": 85.0, "infrastructure": 70.0, "strategic_value": 60.0,
        },
        "prov_ardenne_est": {
            "name": "Marche d'Ardenne",
            "culture": "ardennic",
            "religion": "rationalism",
            "terrain": "hills",
            "climate": {"temperature_c": 12.0, "rainfall_mm": 900.0, "seasonality": 0.50},
            "pop_total": 140000, "pop_urban": 40000, "pop_rural": 100000,
            "resources": [{"resource": "coal", "abundance": 60.0, "reserve": 150000.0}],
            "owner": "ardenne", "controller": "ardenne", "capital_of": None,
            "fortification": 3, "unrest": 8.0, "supply": 75.0, "infrastructure": 65.0, "strategic_value": 75.0,
        },
        "prov_ardenne_val": {
            "name": "Val d'Ardenne",
            "culture": "ardennic",
            "religion": "rationalism",
            "terrain": "plains",
            "climate": {"temperature_c": 15.0, "rainfall_mm": 800.0, "seasonality": 0.40},
            "pop_total": 320000, "pop_urban": 180000, "pop_rural": 140000,
            "resources": [{"resource": "grain", "abundance": 90.0, "reserve": 350000.0}],
            "owner": "ardenne", "controller": "ardenne", "capital_of": "ardenne",
            "fortification": 2, "unrest": 4.0, "supply": 95.0, "infrastructure": 85.0, "strategic_value": 90.0,
        },
        "prov_ardenne_sud": {
            "name": "Bassin de Fer",
            "culture": "ardennic",
            "religion": "rationalism",
            "terrain": "urban",
            "climate": {"temperature_c": 13.5, "rainfall_mm": 820.0, "seasonality": 0.42},
            "pop_total": 210000, "pop_urban": 110000, "pop_rural": 100000,
            "resources": [{"resource": "iron_ore", "abundance": 85.0, "reserve": 500000.0}],
            "owner": "ardenne", "controller": "ardenne", "capital_of": None,
            "fortification": 2, "unrest": 6.0, "supply": 80.0, "infrastructure": 75.0, "strategic_value": 85.0,
        },

        # KESH (Nord-Est)
        "prov_kesh_marche": {
            "name": "Garde de l'Ouest",
            "culture": "keshi",
            "religion": "martial_order",
            "terrain": "hills",
            "climate": {"temperature_c": 11.5, "rainfall_mm": 650.0, "seasonality": 0.55},
            "pop_total": 150000, "pop_urban": 50000, "pop_rural": 100000,
            "resources": [{"resource": "iron_ore", "abundance": 70.0, "reserve": 300000.0}],
            "owner": "kesh", "controller": "kesh", "capital_of": None,
            "fortification": 4, "unrest": 12.0, "supply": 70.0, "infrastructure": 60.0, "strategic_value": 80.0,
        },
        "prov_kesh_steppe": {
            "name": "Grande Steppe",
            "culture": "keshi",
            "religion": "martial_order",
            "terrain": "steppe",
            "climate": {"temperature_c": 13.0, "rainfall_mm": 450.0, "seasonality": 0.65},
            "pop_total": 130000, "pop_urban": 30000, "pop_rural": 100000,
            "resources": [{"resource": "coal", "abundance": 75.0, "reserve": 250000.0}],
            "owner": "kesh", "controller": "kesh", "capital_of": None,
            "fortification": 1, "unrest": 10.0, "supply": 65.0, "infrastructure": 55.0, "strategic_value": 65.0,
        },
        "prov_kesh_citadelle": {
            "name": "Citadelle de Kesh",
            "culture": "keshi",
            "religion": "martial_order",
            "terrain": "hills",
            "climate": {"temperature_c": 12.0, "rainfall_mm": 550.0, "seasonality": 0.60},
            "pop_total": 350000, "pop_urban": 210000, "pop_rural": 140000,
            "resources": [{"resource": "iron_ore", "abundance": 90.0, "reserve": 450000.0}],
            "owner": "kesh", "controller": "kesh", "capital_of": "kesh",
            "fortification": 5, "unrest": 8.0, "supply": 90.0, "infrastructure": 80.0, "strategic_value": 95.0,
        },
        "prov_kesh_collines": {
            "name": "Massif de Karkov",
            "culture": "keshi",
            "religion": "martial_order",
            "terrain": "mountains",
            "climate": {"temperature_c": 9.5, "rainfall_mm": 600.0, "seasonality": 0.70},
            "pop_total": 170000, "pop_urban": 60000, "pop_rural": 110000,
            "resources": [{"resource": "rare_earths", "abundance": 80.0, "reserve": 120000.0}],
            "owner": "kesh", "controller": "kesh", "capital_of": None,
            "fortification": 3, "unrest": 9.0, "supply": 75.0, "infrastructure": 65.0, "strategic_value": 85.0,
        },

        # VARENNE (Sud-Ouest)
        "prov_varenne_marais": {
            "name": "Basses Lagunes",
            "culture": "varennois",
            "religion": "pragmatism",
            "terrain": "swamp",
            "climate": {"temperature_c": 16.5, "rainfall_mm": 1100.0, "seasonality": 0.35},
            "pop_total": 190000, "pop_urban": 80000, "pop_rural": 110000,
            "resources": [{"resource": "grain", "abundance": 65.0, "reserve": 180000.0}],
            "owner": "varenne", "controller": "varenne", "capital_of": None,
            "fortification": 1, "unrest": 7.0, "supply": 85.0, "infrastructure": 75.0, "strategic_value": 65.0,
        },
        "prov_varenne_plaine": {
            "name": "Plaines Dorées",
            "culture": "varennois",
            "religion": "pragmatism",
            "terrain": "plains",
            "climate": {"temperature_c": 17.0, "rainfall_mm": 950.0, "seasonality": 0.38},
            "pop_total": 240000, "pop_urban": 90000, "pop_rural": 150000,
            "resources": [{"resource": "grain", "abundance": 95.0, "reserve": 400000.0}],
            "owner": "varenne", "controller": "varenne", "capital_of": None,
            "fortification": 1, "unrest": 4.0, "supply": 90.0, "infrastructure": 85.0, "strategic_value": 80.0,
        },
        "prov_varenne_port": {
            "name": "Port-Varenne",
            "culture": "varennois",
            "religion": "pragmatism",
            "terrain": "coast",
            "climate": {"temperature_c": 17.5, "rainfall_mm": 1050.0, "seasonality": 0.32},
            "pop_total": 420000, "pop_urban": 280000, "pop_rural": 140000,
            "resources": [{"resource": "oil", "abundance": 75.0, "reserve": 320000.0}],
            "owner": "varenne", "controller": "varenne", "capital_of": "varenne",
            "fortification": 2, "unrest": 5.0, "supply": 95.0, "infrastructure": 90.0, "strategic_value": 95.0,
        },
        "prov_varenne_rive": {
            "name": "Rive Marchande",
            "culture": "varennois",
            "religion": "pragmatism",
            "terrain": "plains",
            "climate": {"temperature_c": 16.0, "rainfall_mm": 980.0, "seasonality": 0.40},
            "pop_total": 200000, "pop_urban": 90000, "pop_rural": 110000,
            "resources": [{"resource": "coal", "abundance": 70.0, "reserve": 200000.0}],
            "owner": "varenne", "controller": "varenne", "capital_of": None,
            "fortification": 2, "unrest": 6.0, "supply": 85.0, "infrastructure": 80.0, "strategic_value": 75.0,
        },

        # ORLUNE (Sud-Est)
        "prov_orlune_temple": {
            "name": "Sanctuaire d'Orlune",
            "culture": "orlunian",
            "religion": "mysticism",
            "terrain": "plains",
            "climate": {"temperature_c": 12.5, "rainfall_mm": 780.0, "seasonality": 0.48},
            "pop_total": 290000, "pop_urban": 170000, "pop_rural": 120000,
            "resources": [{"resource": "rare_earths", "abundance": 85.0, "reserve": 150000.0}],
            "owner": "orlune", "controller": "orlune", "capital_of": "orlune",
            "fortification": 3, "unrest": 6.0, "supply": 90.0, "infrastructure": 80.0, "strategic_value": 90.0,
        },
        "prov_orlune_pics": {
            "name": "Pics Célestes",
            "culture": "orlunian",
            "religion": "mysticism",
            "terrain": "mountains",
            "climate": {"temperature_c": 7.0, "rainfall_mm": 850.0, "seasonality": 0.62},
            "pop_total": 120000, "pop_urban": 30000, "pop_rural": 90000,
            "resources": [{"resource": "rare_earths", "abundance": 90.0, "reserve": 220000.0}],
            "owner": "orlune", "controller": "orlune", "capital_of": None,
            "fortification": 4, "unrest": 5.0, "supply": 65.0, "infrastructure": 60.0, "strategic_value": 85.0,
        },
        "prov_orlune_vallee": {
            "name": "Vallée des Moines",
            "culture": "orlunian",
            "religion": "mysticism",
            "terrain": "hills",
            "climate": {"temperature_c": 14.0, "rainfall_mm": 720.0, "seasonality": 0.50},
            "pop_total": 180000, "pop_urban": 60000, "pop_rural": 120000,
            "resources": [{"resource": "grain", "abundance": 80.0, "reserve": 220000.0}],
            "owner": "orlune", "controller": "orlune", "capital_of": None,
            "fortification": 2, "unrest": 5.0, "supply": 80.0, "infrastructure": 70.0, "strategic_value": 70.0,
        },
        "prov_orlune_sud": {
            "name": "Plateau des Flammes",
            "culture": "orlunian",
            "religion": "mysticism",
            "terrain": "desert",
            "climate": {"temperature_c": 16.0, "rainfall_mm": 500.0, "seasonality": 0.55},
            "pop_total": 160000, "pop_urban": 50000, "pop_rural": 110000,
            "resources": [{"resource": "oil", "abundance": 85.0, "reserve": 450000.0}],
            "owner": "orlune", "controller": "orlune", "capital_of": None,
            "fortification": 2, "unrest": 7.0, "supply": 75.0, "infrastructure": 70.0, "strategic_value": 85.0,
        },
    }

    provinces_full_list = []
    for pid, geom in provinces_geom.items():
        spec = province_specs[pid]
        p_obj = {
            "id": pid,
            "name": spec["name"],
            "polygon": geom["polygon"],
            "center": geom["center"],
            "area_blocks": geom["area_blocks"],
            "neighbors": geom["neighbors"],
            "border_ids": geom["border_ids"],
            "terrain": spec["terrain"],
            "climate": spec["climate"],
            "population": {
                "total": spec["pop_total"],
                "urban": spec["pop_urban"],
                "rural": spec["pop_rural"],
                "growth_rate": 0.005,
                "density_per_km2": 45.0,
            },
            "culture": {
                "primary": spec["culture"],
                "minorities": [],
            },
            "religion": {
                "primary": spec["religion"],
                "minorities": [],
            },
            "resources": spec["resources"],
            "owner": spec["owner"],
            "controller": spec["controller"],
            "fortification": spec["fortification"],
            "unrest": spec["unrest"],
            "supply": spec["supply"],
            "infrastructure": spec["infrastructure"],
            "strategic_value": spec["strategic_value"],
            "victory_points": 10.0 if spec["capital_of"] else 5.0,
            "claims": [],
            "capital_of": spec["capital_of"],
            "tags": ["scenario_province"],
        }
        provinces_full_list.append(p_obj)
        (DATA_DIR / "provinces" / f"{pid}.json").write_text(json.dumps(p_obj, indent=2, ensure_ascii=False), encoding="utf-8")

    provinces_full_list.sort(key=lambda p: p["id"])

    # 14. Nations
    nations = [
        {
            "schema_version": 1,
            "id": "ardenne",
            "name": "République d'Ardenne",
            "adjective": "ardennois",
            "short_name": "Ardenne",
            "color": "#3A7BD5",
            "flag": {"palette": ["#3A7BD5", "#F2F4F8"], "pattern": "horizontal_bands", "motto": "Liberté, Raison, Solidarité"},
            "capital": "prov_ardenne_val",
            "provinces": ["prov_ardenne_est", "prov_ardenne_nord", "prov_ardenne_sud", "prov_ardenne_val"],
            "government": "republic",
            "ideology": "civic_republicanism",
            "culture": "ardennic",
            "religion": "rationalism",
            "language": "fr_FR",
            "leader": {"name": "Mira Valen", "title": "Présidente", "age": 47, "traits": ["prudente", "démocrate", "réformatrice"]},
            "population": {"total": 850000, "workforce": 450000, "conscription_pool": 95000, "growth_rate": 0.005, "urbanization": 0.47},
            "economy": {
                "gdp": 1350000000.0,
                "treasury": 280000000.0,
                "tax_rate": 0.24,
                "inflation": 0.02,
                "unemployment": 0.05,
                "trade_balance": 15000000.0,
                "debt": 80000000.0,
                "sectors": {"agriculture": 0.15, "industry": 0.35, "services": 0.40, "technology": 0.10},
                "resources": {},
            },
            "military": {
                "manpower": 28000,
                "army_size": 20000,
                "readiness": 65.0,
                "morale": 70.0,
                "doctrine": "defensive",
                "missile_stockpile": {},
            },
            "stability": 75.0,
            "legitimacy": 78.0,
            "prestige": 45.0,
            "war_exhaustion": 0.0,
            "unrest": 5.0,
            "technology": {"era": 2, "researched": ["tech_steel_metallurgy", "tech_rail_logistics"], "research_points": 120.0},
            "ai_profile": {
                "archetype": "caretaker",
                "aggression": 0.2,
                "caution": 0.8,
                "honor": 0.8,
                "greed": 0.3,
                "revanchism": 0.1,
                "isolationism": 0.3,
                "opportunism": 0.2,
                "loyalty": 0.8,
                "risk_tolerance": 0.3,
                "patience": 0.8,
                "war_threshold": 0.75,
                "peace_threshold": 0.30,
                "priorities": {"economy": 0.8, "military": 0.4, "diplomacy": 0.8, "expansion": 0.1, "stability": 0.9, "technology": 0.7},
                "red_lines": ["capital_occupied", "core_occupied"],
            },
            "claims": [], "subjects": [], "overlord": None, "allies": [], "wars": [], "revolutions": [], "armies": [], "missiles": [],
            "is_playable": True, "tags": ["democratic"],
        },
        {
            "schema_version": 1,
            "id": "kesh",
            "name": "Autocratie de Kesh",
            "adjective": "keshi",
            "short_name": "Kesh",
            "color": "#E8590C",
            "flag": {"palette": ["#E8590C", "#212529"], "pattern": "cross", "motto": "Par le Fer et le Serment"},
            "capital": "prov_kesh_citadelle",
            "provinces": ["prov_kesh_citadelle", "prov_kesh_collines", "prov_kesh_marche", "prov_kesh_steppe"],
            "government": "autocracy",
            "ideology": "militarist_autocracy",
            "culture": "keshi",
            "religion": "martial_order",
            "language": "fr_FR",
            "leader": {"name": "Goran Thorne", "title": "Généralissime", "age": 52, "traits": ["intransigeant", "stratège", "audacieux"]},
            "population": {"total": 800000, "workforce": 410000, "conscription_pool": 120000, "growth_rate": 0.004, "urbanization": 0.43},
            "economy": {
                "gdp": 1150000000.0,
                "treasury": 180000000.0,
                "tax_rate": 0.28,
                "inflation": 0.04,
                "unemployment": 0.06,
                "trade_balance": -10000000.0,
                "debt": 120000000.0,
                "sectors": {"agriculture": 0.12, "industry": 0.50, "services": 0.28, "technology": 0.10},
                "resources": {},
            },
            "military": {
                "manpower": 45000,
                "army_size": 35000,
                "readiness": 85.0,
                "morale": 80.0,
                "doctrine": "attrition",
                "missile_stockpile": {"missile_short_range": 6},
            },
            "stability": 68.0,
            "legitimacy": 70.0,
            "prestige": 55.0,
            "war_exhaustion": 0.0,
            "unrest": 9.0,
            "technology": {"era": 2, "researched": ["tech_steel_metallurgy"], "research_points": 80.0},
            "ai_profile": {
                "archetype": "warlord",
                "aggression": 0.85,
                "caution": 0.3,
                "honor": 0.4,
                "greed": 0.7,
                "revanchism": 0.75,
                "isolationism": 0.1,
                "opportunism": 0.8,
                "loyalty": 0.4,
                "risk_tolerance": 0.8,
                "patience": 0.4,
                "war_threshold": 0.45,
                "peace_threshold": 0.55,
                "priorities": {"economy": 0.5, "military": 0.9, "diplomacy": 0.3, "expansion": 0.8, "stability": 0.6, "technology": 0.6},
                "red_lines": ["capital_occupied"],
            },
            "claims": [], "subjects": [], "overlord": None, "allies": [], "wars": [], "revolutions": [], "armies": [], "missiles": [],
            "is_playable": True, "tags": ["militarist"],
        },
        {
            "schema_version": 1,
            "id": "varenne",
            "name": "Ligue marchande de Varenne",
            "adjective": "varennois",
            "short_name": "Varenne",
            "color": "#12B886",
            "flag": {"palette": ["#12B886", "#FFD43B"], "pattern": "vertical_bands", "motto": "Flots Prospères, Pacte Éternel"},
            "capital": "prov_varenne_port",
            "provinces": ["prov_varenne_marais", "prov_varenne_plaine", "prov_varenne_port", "prov_varenne_rive"],
            "government": "oligarchy",
            "ideology": "commercial_oligarchy",
            "culture": "varennois",
            "religion": "pragmatism",
            "language": "fr_FR",
            "leader": {"name": "Alvise Contarini", "title": "Doge", "age": 59, "traits": ["négociateur", "richissime", "diplomate"]},
            "population": {"total": 1050000, "workforce": 560000, "conscription_pool": 80000, "growth_rate": 0.007, "urbanization": 0.52},
            "economy": {
                "gdp": 1650000000.0,
                "treasury": 450000000.0,
                "tax_rate": 0.18,
                "inflation": 0.02,
                "unemployment": 0.04,
                "trade_balance": 45000000.0,
                "debt": 50000000.0,
                "sectors": {"agriculture": 0.18, "industry": 0.22, "services": 0.50, "technology": 0.10},
                "resources": {},
            },
            "military": {
                "manpower": 22000,
                "army_size": 16000,
                "readiness": 55.0,
                "morale": 65.0,
                "doctrine": "naval",
                "missile_stockpile": {},
            },
            "stability": 80.0,
            "legitimacy": 74.0,
            "prestige": 60.0,
            "war_exhaustion": 0.0,
            "unrest": 5.0,
            "technology": {"era": 2, "researched": ["tech_rail_logistics"], "research_points": 150.0},
            "ai_profile": {
                "archetype": "merchant",
                "aggression": 0.15,
                "caution": 0.7,
                "honor": 0.6,
                "greed": 0.85,
                "revanchism": 0.1,
                "isolationism": 0.2,
                "opportunism": 0.6,
                "loyalty": 0.6,
                "risk_tolerance": 0.4,
                "patience": 0.7,
                "war_threshold": 0.80,
                "peace_threshold": 0.25,
                "priorities": {"economy": 0.9, "military": 0.3, "diplomacy": 0.8, "expansion": 0.2, "stability": 0.8, "technology": 0.7},
                "red_lines": ["capital_occupied", "trade_embargo"],
            },
            "claims": [], "subjects": [], "overlord": None, "allies": [], "wars": [], "revolutions": [], "armies": [], "missiles": [],
            "is_playable": True, "tags": ["mercantile"],
        },
        {
            "schema_version": 1,
            "id": "orlune",
            "name": "Sanctuaire d'Orlune",
            "adjective": "orlunien",
            "short_name": "Orlune",
            "color": "#9C36B5",
            "flag": {"palette": ["#9C36B5", "#F8F9FA"], "pattern": "starfield", "motto": "Sous le Regard des Astres"},
            "capital": "prov_orlune_temple",
            "provinces": ["prov_orlune_pics", "prov_orlune_sud", "prov_orlune_temple", "prov_orlune_vallee"],
            "government": "theocracy",
            "ideology": "devotional_theocracy",
            "culture": "orlunian",
            "religion": "mysticism",
            "language": "fr_FR",
            "leader": {"name": "Elysia Solas", "title": "Archonte Suprême", "age": 44, "traits": ["mystique", "érudite", "déterminée"]},
            "population": {"total": 750000, "workforce": 380000, "conscription_pool": 85000, "growth_rate": 0.003, "urbanization": 0.41},
            "economy": {
                "gdp": 1050000000.0,
                "treasury": 220000000.0,
                "tax_rate": 0.22,
                "inflation": 0.015,
                "unemployment": 0.05,
                "trade_balance": 5000000.0,
                "debt": 40000000.0,
                "sectors": {"agriculture": 0.20, "industry": 0.25, "services": 0.35, "technology": 0.20},
                "resources": {},
            },
            "military": {
                "manpower": 25000,
                "army_size": 18000,
                "readiness": 65.0,
                "morale": 85.0,
                "doctrine": "defensive",
                "missile_stockpile": {},
            },
            "stability": 82.0,
            "legitimacy": 85.0,
            "prestige": 50.0,
            "war_exhaustion": 0.0,
            "unrest": 4.0,
            "technology": {"era": 2, "researched": ["tech_radio_communications"], "research_points": 180.0},
            "ai_profile": {
                "archetype": "ideologue",
                "aggression": 0.35,
                "caution": 0.6,
                "honor": 0.7,
                "greed": 0.3,
                "revanchism": 0.4,
                "isolationism": 0.5,
                "opportunism": 0.4,
                "loyalty": 0.7,
                "risk_tolerance": 0.5,
                "patience": 0.7,
                "war_threshold": 0.65,
                "peace_threshold": 0.40,
                "priorities": {"economy": 0.5, "military": 0.5, "diplomacy": 0.6, "expansion": 0.3, "stability": 0.9, "technology": 0.8},
                "red_lines": ["capital_occupied", "core_occupied"],
            },
            "claims": [], "subjects": [], "overlord": None, "allies": [], "wars": [], "revolutions": [], "armies": [], "missiles": [],
            "is_playable": True, "tags": ["theocratic"],
        },
    ]

    for nat in nations:
        (DATA_DIR / "nations" / f"{nat['id']}.json").write_text(json.dumps(nat, indent=2, ensure_ascii=False), encoding="utf-8")

    # 15. Scénario complet 'Arc des Frontières'
    # Diplomatie initiale (paires triées lexicale a < b)
    relations = [
        # ardenne < kesh
        {
            "a": "ardenne",
            "b": "kesh",
            "opinion": -55.0,
            "trust": 25.0,
            "fear": 45.0,
            "trade_dependence": 0.05,
            "border_tension": 60.0,
            "status": "hostile",
        },
        # ardenne < orlune
        {
            "a": "ardenne",
            "b": "orlune",
            "opinion": 15.0,
            "trust": 50.0,
            "fear": 10.0,
            "trade_dependence": 0.15,
            "border_tension": 10.0,
            "status": "neutral",
        },
        # ardenne < varenne
        {
            "a": "ardenne",
            "b": "varenne",
            "opinion": 40.0,
            "trust": 65.0,
            "fear": 10.0,
            "trade_dependence": 0.30,
            "border_tension": 15.0,
            "status": "friendly",
        },
        # kesh < orlune
        {
            "a": "kesh",
            "b": "orlune",
            "opinion": -20.0,
            "trust": 35.0,
            "fear": 30.0,
            "trade_dependence": 0.10,
            "border_tension": 40.0,
            "status": "neutral",
        },
        # kesh < varenne
        {
            "a": "kesh",
            "b": "varenne",
            "opinion": 5.0,
            "trust": 40.0,
            "fear": 20.0,
            "trade_dependence": 0.25,
            "border_tension": 25.0,
            "status": "neutral",
        },
        # orlune < varenne
        {
            "a": "orlune",
            "b": "varenne",
            "opinion": 25.0,
            "trust": 55.0,
            "fear": 15.0,
            "trade_dependence": 0.20,
            "border_tension": 15.0,
            "status": "friendly",
        },
    ]

    scenario = {
        "schema_version": 1,
        "id": "arc_des_frontieres",
        "name": "Arc des Frontières",
        "description": "Scénario officiel de Civilisations : quatre puissances rivales s'affrontent pour le contrôle d'un carrefour géopolitique majeur.",
        "start_date": {"year": 2026, "month": 1, "day": 1},
        "seed": 424242,
        "world": {
            "schema_version": 1,
            "seed": 424242,
            "size": {"width": 1024, "height": 1024},
            "sea_level": 62,
            "generation": {
                "algorithm": "hand_authored_v1",
                "version": 1,
                "border_noise": 0.35,
                "coast_complexity": 0.55,
            },
            "provinces": provinces_full_list,
            "borders": borders,
            "regions": [
                {
                    "id": "vallees_du_nord",
                    "name": "Vallées du Nord",
                    "kind": "subcontinent",
                    "provinces": ["prov_ardenne_est", "prov_ardenne_nord", "prov_ardenne_sud", "prov_ardenne_val"],
                    "climate": "temperate",
                },
                {
                    "id": "marches_de_lest",
                    "name": "Marches de l'Est",
                    "kind": "subcontinent",
                    "provinces": ["prov_kesh_citadelle", "prov_kesh_collines", "prov_kesh_marche", "prov_kesh_steppe"],
                    "climate": "semi_arid",
                },
                {
                    "id": "bassin_maritime",
                    "name": "Bassin Maritime",
                    "kind": "subcontinent",
                    "provinces": ["prov_varenne_marais", "prov_varenne_plaine", "prov_varenne_port", "prov_varenne_rive"],
                    "climate": "oceanic",
                },
                {
                    "id": "hauts_sanctuaires",
                    "name": "Hauts Sanctuaires",
                    "kind": "subcontinent",
                    "provinces": ["prov_orlune_pics", "prov_orlune_sud", "prov_orlune_temple", "prov_orlune_vallee"],
                    "climate": "continental",
                },
            ],
            "straits": [],
            "climate_zones": [
                {
                    "id": "temperate_north",
                    "name": "Zone tempérée nordique",
                    "temperature_range": {"min": 5.0, "max": 25.0},
                    "rainfall_range": {"min": 400.0, "max": 1200.0},
                    "biomes": ["forest", "plains"],
                }
            ],
        },
        "nations": nations,
        "diplomacy": {
            "schema_version": 1,
            "relations": relations,
            "treaties": [],
            "organizations": [],
            "sanctions": [],
            "guarantees": [],
            "espionage_operations": [],
            "trade_routes": [],
        },
        "wars": [],
        "revolutions": [],
        "armies": [],
        "missiles_in_flight": [],
        "player_nation": "ardenne",
        "difficulty": "normal",
        "tags": ["official", "arc_des_frontieres"],
    }

    (DATA_DIR / "scenarios" / "arc_des_frontieres.json").write_text(json.dumps(scenario, indent=2, ensure_ascii=False), encoding="utf-8")

    # 16. Config TOML
    config_toml = """schema_version = 1

[simulation]
ticks_per_day = 2400
ai_interval_ticks = 100
max_events_per_tick = 32
seed = 424242
start_date = { year = 2026, month = 1, day = 1 }
autosave_days = 30
max_command_history = 4096

[world]
map_width = 1024
map_height = 1024
province_count = 16
sea_level = 62
border_style = "warped"
border_noise = 0.35
coast_complexity = 0.55
min_province_area = 1024
max_province_neighbors = 8

[ai]
difficulty = "normal"
max_war_fronts = 8
max_diplomacy_actions_per_tick = 8
max_armies_per_nation = 24
allow_ai_nukes = false
ai_reaction_delay_ticks = 200

[war]
max_active_wars = 16
battle_interval_ticks = 200
occupation_speed = 1.0
attrition_rate = 0.02
peace_offer_cooldown_days = 14
allow_forced_regime_change = true

[missiles]
max_in_flight = 64
flight_ticks_per_chunk = 20
interception_enabled = true
interception_chance_scale = 1.0
show_trajectory = true
cinematic_impact = true

[ui]
theme = "dark"
map_zoom_min = 0.5
map_zoom_max = 4.0
show_province_names = true
show_borders = true
show_armies = true
show_missiles = true

[multiplayer]
enabled = false
max_players = 4
sync_rate_ticks = 20
"""
    (DATA_DIR / "config.toml").write_text(config_toml, encoding="utf-8")

    # 17. Manifeste du pack de données
    manifest = {
        "schema_version": 1,
        "id": "civ",
        "version": "0.1.0",
        "game": "26.3",
        "horizon": "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0",
        "name": "Civilisations — Arc des Frontières",
        "description": "Pack de données officiel pour Civilisations avec le scénario 'Arc des Frontières'.",
        "authors": ["L'Équipe Civilisations"],
        "license": "CC-BY-SA-4.0",
        "content": {
            "config": "config.toml",
            "scenarios": ["scenarios/arc_des_frontieres.json"],
            "nations": [
                "nations/ardenne.json",
                "nations/kesh.json",
                "nations/varenne.json",
                "nations/orlune.json",
            ],
            "provinces": [f"provinces/{p['id']}.json" for p in provinces_full_list],
            "cultures": [f"cultures/{c['id']}.json" for c in cultures],
            "religions": [f"religions/{r['id']}.json" for r in religions],
            "ideologies": [f"ideologies/{i['id']}.json" for i in ideologies],
            "governments": [f"governments/{g['id']}.json" for g in governments],
            "resources": [f"resources/{res['id']}.json" for res in resources],
            "technologies": [f"technologies/{t['id']}.json" for t in technologies],
            "units": [f"units/{u['id']}.json" for u in units],
            "missiles": [f"missiles/{m['id']}.json" for m in missiles],
            "events": [f"events/{e['id']}.json" for e in events],
            "names": [f"names/{n['id']}.json" for n in names_pools],
            "localization": [
                "localization/fr_fr.json",
                "localization/en_us.json",
            ],
        },
        "dependencies": [],
        "load_after": [],
    }
    (DATA_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8")

    # 18. Assets de production originaux sous assets/civ/
    (ASSETS_DIR / "textures" / "flags").mkdir(parents=True, exist_ok=True)
    (ASSETS_DIR / "textures" / "map").mkdir(parents=True, exist_ok=True)
    (ASSETS_DIR / "textures" / "ui").mkdir(parents=True, exist_ok=True)
    (ASSETS_DIR / "sounds").mkdir(parents=True, exist_ok=True)
    (ASSETS_DIR / "particles").mkdir(parents=True, exist_ok=True)
    (ASSETS_DIR / "lang").mkdir(parents=True, exist_ok=True)

    # Icône et drapeaux PNG originaux
    (ASSETS_DIR / "icon.png").write_bytes(create_png(64, 64, (58, 123, 213)))
    (ASSETS_DIR / "textures" / "flags" / "ardenne.png").write_bytes(create_png(64, 40, (58, 123, 213)))
    (ASSETS_DIR / "textures" / "flags" / "kesh.png").write_bytes(create_png(64, 40, (232, 89, 12)))
    (ASSETS_DIR / "textures" / "flags" / "varenne.png").write_bytes(create_png(64, 40, (18, 184, 134)))
    (ASSETS_DIR / "textures" / "flags" / "orlune.png").write_bytes(create_png(64, 40, (156, 54, 181)))

    # Fichiers de localisation lang/
    (ASSETS_DIR / "lang" / "fr_fr.json").write_text(json.dumps(loc_fr, indent=2, ensure_ascii=False), encoding="utf-8")
    (ASSETS_DIR / "lang" / "en_us.json").write_text(json.dumps(loc_en, indent=2, ensure_ascii=False), encoding="utf-8")

    # README assets mis à jour
    assets_readme = """# Assets — Civilisations

Tous les assets de ce dossier sont **originaux** et sous licence **CC-BY-SA-4.0**.

## Organisation

```text
assets/civ/
├─ icon.png
├─ textures/flags/
│   ├─ ardenne.png
│   ├─ kesh.png
│   ├─ varenne.png
│   └─ orlune.png
├─ textures/map/
├─ textures/ui/
├─ models/
├─ sounds/
├─ particles/
└─ lang/
    ├─ fr_fr.json
    └─ en_us.json
```

## Conformité

- 100% original, aucun asset externe non sous licence libre.
- Pas de symboles réels, de drapeaux réels ou d'entités du monde réel.
"""
    (ASSETS_DIR / "README.md").write_text(assets_readme, encoding="utf-8")

    print(f"Pack de données complet généré avec succès dans {DATA_DIR}.")


if __name__ == "__main__":
    generate_all()
