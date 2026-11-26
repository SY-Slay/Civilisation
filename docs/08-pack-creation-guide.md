# 08 — Guide de création de packs de données

## Introduction

Le mod **Civilisations** sépare strictement son moteur géopolitique de son contenu narratif et géographique. Tout le contenu jouable (nations, provinces, technologies, missiles, événements, cultures, idéologies et scénarios) est défini sous forme de **packs de données déclaratifs JSON**.

Ce guide détaille la structure attendue, les règles de validation strictes et les outils de contrôle qualité pour concevoir un pack de données officiel ou communautaire.

---

## 1. Arborescence type d'un pack de données

Un pack de données valide réside dans un dossier (par exemple `data/civ/` pour le pack officiel) et adopte la structure suivante :

```text
mon_pack/
├── manifest.json              # Métadonnées, version et licence du pack
├── config.toml                # Paramètres de simulation par défaut
├── cultures/                  # Fichiers culture (.json)
├── religions/                 # Fichiers religion (.json)
├── ideologies/                # Fichiers idéologie (.json)
├── governments/               # Fichiers gouvernement (.json)
├── resources/                 # Ressources économiques et stratégiques (.json)
├── technologies/              # Arbre technologique et prérequis (.json)
├── units/                     # Définitions des corps d'armée (.json)
├── missiles/                  # Définitions des vecteurs balistiques (.json)
├── events/                    # Événements scriptés, cooldowns et effets (.json)
├── names/                     # Réservoirs de noms pour l'IA et les dirigeants (.json)
├── localization/              # Dictionnaires de localisation fr_fr.json, en_us.json
├── nations/                   # Définitions des nations participantes (.json)
├── provinces/                 # Géométrie polygonale et climat des provinces (.json)
└── scenarios/                 # Scénarios assemblant monde, nations et diplomatie (.json)
```

---

## 2. Le fichier `manifest.json`

Le manifeste décrit l'identité du pack selon le schéma `schemas/data-pack.schema.json` :

```json
{
  "schema_version": 1,
  "id": "mon_pack_geopolitique",
  "version": "1.0.0",
  "name": "Nouveau Monde Fictif",
  "authors": ["Auteur Original"],
  "license": "CC-BY-SA-4.0",
  "description": "Un scénario géopolitique original opposant quatre coalitions insulaires.",
  "dependencies": []
}
```

> [!IMPORTANT]
> Pour garantir la pérennité et la diffusion libre au sein de la communauté, tous les packs de données et assets associés doivent être publiés sous une licence ouverte compatible **CC-BY-SA-4.0**.

---

## 3. Politique de contenu 100% fictionnelle

Tout pack de données destiné au mod doit respecter rigoureusement la charte documentée dans `docs/00-content-policy.md` :

- **Aucun pays réel :** Interdiction stricte de reproduire des États, nations historiques ou actuels (ex. France, Russie, États-Unis, Chine, etc.).
- **Aucun personnage historique ou politique réel :** Les réservoirs de noms et dirigeants doivent être entièrement inventés.
- **Aucune géographie ou frontière réelle :** Les coordonnées de blocs et contours de provinces ne doivent reproduire aucun tracé frontalier du monde réel.
- **Aucun système d'armement réel :** Ne pas utiliser de désignations d'armes réelles (ex. Kalachnikov, Tomahawk, F-35) ; utiliser des concepts génériques ou stylisés (ex. *Batterie d'Artillerie Lourde*, *Missile de Croisière Tactique*).
- **Thèmes interdits :** Aucun génocide, haine raciale, discrimination réelle ou glorification de régimes totalitaires historiques.

---

## 4. Règles géométriques et topologiques des provinces

Les provinces forment le support spatial de la simulation géopolitique. Pour garantir la cohérence mathématique et la fluidité du rendu 3D dans Horizon, chaque province (`province.schema.json`) doit respecter les contraintes suivantes :

1. **Polygone simple et non-croisé :**
   - Le contour extérieur (`exterior`) doit comporter au moins 3 sommets (en coordonnées entières du monde Horizon).
   - Les arêtes ne doivent pas s'auto-intersecter.
   - Les trous intérieurs éventuels (`holes`) doivent être strictement inclus dans le polygone extérieur sans se recouper entre eux.

2. **Superficie minimale :**
   - L'aire plane calculée via la formule du lacet (Shoelace) doit être strictement supérieure à **1024 blocs carrés** (32x32 blocs).

3. **Frontières naturelles et irrégulières :**
   - Afin d'éviter les lignes artificielles ou géométriquement plates, **aucun segment rectiligne ne doit dépasser 160 blocs de longueur**.
   - Ajoutez des sommets intermédiaires le long des cours d'eau, des crêtes montagneuses ou des vallées.

4. **Réciprocité stricte des frontières partagées :**
   - Toute frontière terrestre partagée entre une province $A$ et une province $B$ doit être déclarée avec un `border_id` unique partagé.
   - Si la frontière apparaît dans la liste des frontières du scénario, elle doit référencer exactement deux provinces adjacentes valides.

---

## 5. Arbre technologique et registres

- **Identifiants typés :** Tous les identifiants d'entités (`tech_id`, `unit_id`, `nation_id`, `prov_id`, etc.) doivent respecter le motif regex `^[a-z0-9_]{3,32}$`.
- **Intégrité référentielle :**
  - Tout prérequis (`prerequisites`) d'une technologie doit désigner une technologie existante dans le registre du pack.
  - L'arbre technologique ne doit présenter **aucun cycle** de dépendance.
  - Les ressources requises par une unité ou un missile doivent correspondre à un identifiant défini dans `resources/`.

---

## 6. Outils de validation automatisés

Avant de distribuer ou charger un pack dans le jeu, deux scripts d'assurance qualité doivent obligatoirement être exécutés :

### 1. Validation de la conformité JSON Schema
```bash
python tools/validate_schemas.py
```
Vérifie que les 24 schémas JSON Draft 2020-12 du mod sont syntaxiquement valides et capables d'évaluer les fixtures de test.

### 2. Validation métier et intégrité du pack
```bash
python tools/validate_data.py --pack data/civ
```
Ce script effectue un audit complet :
- Validation de chaque fichier JSON contre son schéma dédié.
- Vérification de l'intégrité référentielle croisée (clés étrangères, relations, IDs).
- Audit géométrique : calcul d'aire, détection d'auto-intersections, longueur des segments < 160 blocs, réciprocité des 24 frontières.
- Audit de conformité de contenu : détection de termes proscrits (noms de pays réels, dirigeants, coordonnées géographiques réelles).

---

## 7. Intégration dans Horizon

Une fois validé, le pack peut être intégré selon deux méthodes :

1. **Intégration directe dans le bundle natif :**
   - Placer les données sous `data/civ/` à la racine du dépôt.
   - Lancer `python tools/package_native.py`. Le dossier `data/civ/` sera automatiquement empaqueté dans `native/civ.hmod`.

2. **Dossier de surcharge d'instance (Data-pack local) :**
   - Déposer votre dossier de pack dans `<instance>/data/civ/` ou `<world_dir>/datapacks/civ/`.
   - Lancer la commande `/civ reload` en console ou en jeu pour charger à chaud le nouveau contenu.
