# 02 — Modèle de données

## Index des schémas

Tous les schémas utilisent **JSON Schema Draft 2020-12**. Les `$id` sont sous `https://civ.invalid/schemas/` ; `civ.invalid` est un domaine réservé à la documentation, aucun appel réseau n'est effectué.

| Fichier | `$id` | Rôle |
|---|---|---|
| `common.schema.json` | `.../common.schema.json` | IDs, textes localisés, dates, vecteurs, polygones, couleurs, modifiers, conditions, effets |
| `index.schema.json` | `.../index.schema.json` | Index ordonné d'un registre de données |
| `data-pack.schema.json` | `.../data-pack.schema.json` | Manifeste d'un pack de données et liste de ses fichiers |
| `config.schema.json` | `.../config.schema.json` | Configuration globale (TOML converti en JSON pour validation) |
| `world.schema.json` | `.../world.schema.json` | Monde, provinces, frontières, régions, détroits, climat |
| `province.schema.json` | `.../province.schema.json` | Province, population, culture, religion, ressources, propriété |
| `culture.schema.json` | `.../culture.schema.json` | Culture fictive, valeurs, traditions, réservoir de noms |
| `religion.schema.json` | `.../religion.schema.json` | Religion fictive, dogmes, tolérance, sites saints |
| `ideology.schema.json` | `.../ideology.schema.json` | Famille idéologique fictive, valeurs, gouvernements préférés |
| `government.schema.json` | `.../government.schema.json` | Forme de gouvernement, légitimité, succession, durée |
| `resource.schema.json` | `.../resource.schema.json` | Ressource économique, catégorie, unité, valeur de base |
| `nation.schema.json` | `.../nation.schema.json` | Nation, gouvernement, économie, armée, technologie, profil IA |
| `diplomacy.schema.json` | `.../diplomacy.schema.json` | Relations, traités, organisations, sanctions, garanties, espionnage |
| `war.schema.json` | `.../war.schema.json` | Guerre, belligérants, objectifs, fronts, batailles, armées, paix |
| `revolution.schema.json` | `.../revolution.schema.json` | Factions, soutien provincial, réponse du gouvernement, issue |
| `missile.schema.json` | `.../missile.schema.json` | Type de missile **ou** instance en vol |
| `technology.schema.json` | `.../technology.schema.json` | Nœud d'arbre technologique |
| `unit.schema.json` | `.../unit.schema.json` | Type d'unité militaire |
| `event.schema.json` | `.../event.schema.json` | Événement narratif, déclencheurs, choix, effets |
| `game-data.schema.json` | `.../game-data.schema.json` | Agrégat éditable technologies + missiles + événements chargé par `civ-core` |
| `names.schema.json` | `.../names.schema.json` | Réservoirs de noms fictifs |
| `localization.schema.json` | `.../localization.schema.json` | Table de traduction |
| `scenario.schema.json` | `.../scenario.schema.json` | Scénario de départ complet |
| `save.schema.json` | `.../save.schema.json` | Sauvegarde complète et compatible |

## Types communs

### Identifiants

- Format : `^[a-z][a-z0-9_]{2,63}$`.
- Un ID est stable, unique dans sa collection et jamais réutilisé après suppression.
- Les IDs sont des chaînes, pas des entiers : une sauvegarde reste lisible et éditable.
- Les paires de relations utilisent `a < b` dans l'ordre lexical pour garantir une seule entrée par paire. Le schéma documente la règle ; `civ-core` la vérifie.

### Textes localisés

`localized_text` accepte :

- une chaîne directe (repli immédiat) ;
- un objet `{ "key": "civ.nation.ardenne.name", "default": "Ardenne" }`.

Les clés suivent `^[a-z0-9_.-]+$`. Le français est la langue de référence ; l'anglais sert de repli.

### Dates et ticks

- `game_date` : `{ year, month, day }`, mois 1–12, jour 1–30. Calendrier fictif simple, sans années bissextiles.
- `tick` : tick de simulation, indépendant du tick Minecraft.
- `config.simulation.ticks_per_day` fixe la conversion. Une sauvegarde enregistre le tick, la date, la date initiale, la difficulté et le nombre de ticks par jour (ces trois derniers champs sont optionnels pour relire les anciennes sauvegardes).
- Aucune règle de jeu ne lit l'horloge système.

### Géométrie

- `vec2` : `[x, z]` en blocs.
- `vec3` : `[x, y, z]`.
- `polygon` : au moins 3 points, fermé implicitement, sens antihoraire.
- Le schéma valide la **forme JSON**. `civ-core` valide la **géométrie** : polygone simple, aire minimale, arêtes partagées, absence de trous, recouvrement du monde.

### Conditions et effets

`condition` et `effect` sont des objets `{ kind, args }` extensibles :

- `kind` est un identifiant stable, interprété par `civ-core` ;
- `args` contient les paramètres propres au kind ;
- un kind inconnu est refusé au chargement, jamais ignoré silencieusement ;
- les effets sont des données, jamais du code exécutable.

Kinds actifs à l'étape 3 :

- conditions : `stability_below`, `stability_above`, `unrest_above`, `legitimacy_below`, `war_exhaustion_above`, `gdp_below`, `treasury_below`, `at_war`, `has_revolution`, `sanctioned`, `research_at_least`, `relation_opinion_below`, `border_tension_above` ;
- effets : `stability`, `legitimacy`, `unrest`, `war_exhaustion`, `prestige`, `treasury`, `gdp`, `manpower`, `army_size`, `research_points`, `relation_opinion`/`opinion`, `border_tension`, `start_revolution`, `province_unrest`.

Un kind inconnu est refusé au chargement de `GameData` ; pour un effet différé déjà présent dans une sauvegarde, il échoue à l'exécution. Il ne doit jamais être ignoré silencieusement. Un effet bilatéral peut fixer sa cible avec `args.nation` et son autre partie avec `args.other` (alias `other_nation`). Sans autre partie explicite, le moteur choisit la relation la plus tendue, puis la plus basse en opinion, avec tie-break par identifiant.

## Monde et provinces

### Modèle territorial

- Une **province** est l'unité atomique de territoire.
- Le monde n'est pas une grille de chunks : c'est un graphe de provinces polygonales.
- Chaque frontière partagée est stockée **une seule fois** dans `world.borders`, avec `a`, `b` et une polyligne.
- `province.border_ids` référence ces frontières ; `province.neighbors` donne les provinces adjacentes.
- Une province a un `owner` (de jure) et un `controller` (de facto). Une occupation change `controller`, pas `owner` tant qu'aucun traité n'est signé.
- Les mers ne sont pas des provinces. Les détroits et routes maritimes sont des objets `straits` séparés ; les opérations navales restent abstraites.

### Frontières irrégulières

La génération et la validation appliquent :

- bruit cohérent multi-échelle ;
- perturbation par le relief, les rivières et les côtes ;
- nombre minimal de sommets sur les longues frontières ;
- ratio de rectitude maximal ;
- arêtes partagées sans décalage ;
- aucune ligne droite sur une distance excessive.

Les fixtures de validation utilisent des polygones simples : elles prouvent le schéma, pas la qualité cartographique.

### Population et contrôle

- `population.total = urban + rural` doit être cohérent ; `civ-core` le vérifie.
- `culture.primary`, `religion.primary`, `resources[].resource` référencent des registres.
- `unrest`, `supply`, `infrastructure`, `strategic_value` sont des pourcentages 0–100.
- `capital_of` pointe vers une nation ; une capitale doit appartenir à cette nation au chargement.

## Nations

Une nation contient :

- identité : nom, adjectif, couleur, drapeau, capitale, provinces ;
- politique : gouvernement, idéologie, culture, religion, dirigeant ;
- démographie : population, main-d'œuvre, réserve de conscription ;
- économie : PIB, trésor, fiscalité, inflation, chômage, secteurs, stockpiles ;
- militaire : manpower, taille d'armée, readiness, morale, doctrine, capacité nucléaire ;
- état : stabilité, légitimité, prestige, usure de guerre, unrest ;
- technologie : ère, technologies recherchées, recherche en cours ;
- IA : archétype, traits 0–1, priorités, seuils de guerre/paix, lignes rouges ;
- relations : revendications, sujets, suzerain, alliances, guerres, révolutions, armées, missiles.

### Profil IA

Les traits sont des nombres 0–1 :

`aggression`, `caution`, `honor`, `greed`, `revanchism`, `isolationism`, `opportunism`, `loyalty`, `risk_tolerance`, `patience`.

L'IA est une **utility AI** déterministe :

1. collecter les objectifs valides ;
2. scorer chaque objectif avec les traits, l'état du monde et les contraintes ;
3. tirer un bruit déterministe par flux ;
4. choisir le meilleur score ;
5. tie-break par ID stable.

Aucun texte n'est généré à l'exécution. Les événements narratifs sont des données.

## Diplomatie

- `relations` : une entrée par paire, `a < b`, avec opinion (−100 à 100), confiance, peur, dépendance commerciale, tension frontalier, statut.
- `treaties` : traité avec parties, termes, dates, statut, violations.
- `organizations` : alliance, bloc commercial, union douanière, pacte de défense, fédération, conseil, cartel, réseau de recherche.
- `sanctions` : coalitions de sanctionnaires, cible, type, sévérité, dates, exemptions.
- `guarantees` : garanties d'indépendance ou de protection.
- `espionage_operations` : opérations d'influence, progression, exposition.
- `trade_routes` : routes, volume, statut, blocus.

Les termes de traité (`term.kind`) sont des kinds interprétés par `civ-core` : céder une province, payer des réparations, démilitariser, ouvrir un marché, reconnaître un gouvernement, libérer un vassal, etc.

## Guerre

- Une guerre a un `casus_belli`, des belligérants, des objectifs, des fronts, des batailles, un score et des offres de paix.
- Chaque belligérant a un camp, un soutien populaire, des pertes, des provinces occupées et une contribution.
- Un front couvre une ou plusieurs provinces, avec momentum, ravitaillement et bilan de batailles.
- Une bataille est résolue avec un flux RNG dédié ; le résultat est stocké, pas recalculé à l'affichage.
- Une armée a une nation, une localisation, des unités, du ravitaillement, du moral, une posture, un général et un éventuel mouvement.
- La paix produit un traité (`diplomacy.treaty`) et peut changer `owner`, `controller`, les vassaux, les réparations ou le gouvernement.

## Révolution

- Une révolution a des causes, des factions, un soutien par province, des provinces contrôlées, une réponse gouvernementale, des interventions étrangères et une issue.
- Les factions ont une idéologie, un soutien, des demandes, une milice, une légitimité et un soutien étranger.
- Le passage `uprising -> civil_war` crée ou réutilise une guerre civile ; les deux systèmes partagent les provinces et les armées.
- Une issue peut être `status_quo`, `reform`, `new_government`, `independence`, `autonomy`, `partition` ou `crushed`.

## Missiles

Le schéma `missile.schema.json` est un `oneOf` :

- **type** : définition de données (`range_km`, vitesse, précision, warhead, coût, temps de production, pénétration, plateformes, effets) ;
- **instance** : missile en vol (propriétaire, lanceur, cible, ticks, statut, trajectoire, intercepteurs, rapport de dégâts).

Le cœur ne connaît ni particules ni sons. Il produit un `DamageReport` abstrait. `civ-horizon` traduit via `ExplosionEffects`.

### Intégration Orbital Laser

- Le rendu par défaut utilise les particules et sons de `civ-horizon`.
- Si Orbital Laser est présent, un adaptateur peut réutiliser ses effets **visuels** s'il expose une API stable.
- Orbital Laser n'a pas de commande de frappe par coordonnées vérifiée : le mod ne doit pas prétendre pouvoir déclencher une frappe orbitale à distance.
- Sans API stable, le rendu de repli reste utilisé.

## Événements

Un événement contient :

- identité, nom, description, type, poids, cooldown, nombre maximal de déclenchements ;
- un trigger : conditions, dates, nations/provinces concernées, guerre/révolution requise, chance ;
- des choix : texte, conditions, effets, poids IA, visibilité ;
- un éventuel choix forcé pour l'IA.

La sélection est stable : liste triée par ID, score pondéré, tirage dans un flux dédié. Un événement déjà tiré ne peut pas réapparaître avant son cooldown. Les effets différés sont stockés dans `event_state.pending` avec leur tick d'application et leur nation cible, afin de survivre à une sauvegarde/rechargement.

## Cultures, religions, idéologies, gouvernements, ressources

Ces registres sont des données de contenu, pas des règles codées en dur :

- `culture` : langue, valeurs, traditions, réservoir de noms, couleur ;
- `religion` : famille, dogmes, tolérance, sites saints ;
- `ideology` : famille, valeurs, gouvernements préférés ;
- `government` : catégorie, source de légitimité, mode de succession, durée du mandat ;
- `resource` : catégorie, unité, valeur de base, caractère stratégique.

`nation.government`, `nation.ideology`, `nation.culture`, `nation.religion`, `province.culture.primary`, `province.religion.primary` et `province.resources[].resource` sont des IDs. `civ-core` vérifie que chaque ID existe dans le registre correspondant avant de charger une partie.

Le manifeste `data-pack.schema.json` déclare les fichiers de ces registres dans `content.cultures`, `content.religions`, `content.ideologies`, `content.governments` et `content.resources`.

## Technologies et unités

- Une technologie a une ère, une catégorie, un coût, un temps de recherche, des prérequis, des effets et des déblocages.
- Une unité a une catégorie, une ère, un coût, un entretien, du manpower, des statistiques, un temps d'entraînement, des prérequis et des counters.
- Les valeurs sont abstraites et équilibrées pour la simulation géopolitique, pas pour un jeu de guerre tactique.

## Scénario et sauvegarde

### Scénario

Un scénario est un point de départ complet :

- monde (provinces, frontières, régions, détroits) ;
- nations avec leurs provinces ;
- diplomatie initiale ;
- guerres, révolutions, armées et missiles éventuels ;
- nation joueuse, difficulté, tags.

### Sauvegarde

Une sauvegarde contient :

- identité et dates ;
- compatibilité (jeu, mod, commit Horizon, empreinte des données) ;
- scénario d'origine, graine, tick, date ;
- monde, nations, diplomatie, guerres, révolutions, armées, missiles en vol ;
- état des événements (actifs, cooldowns, compteurs, effets différés) ;
- état RNG ;
- checksum SHA-256 ;
- métadonnées (nom, nation joueuse, temps de jeu, notes, multijoueur).

Le format est décrit dans `03-save-format.md`.

## Validation à deux niveaux

### Niveau 1 — JSON Schema

- structure, types, énumérations, patterns, bornes ;
- champs requis ;
- références de schéma.

### Niveau 2 — `civ-core`

Implémenté à l'étape 3 pour un scénario et pour `GameData` (technologies, missiles, événements) :

- unicité des IDs ;
- références existantes ;
- réciprocité des voisins ;
- frontières partagées par exactement deux provinces ;
- capitale dans la nation ;
- cohérence population/urbain/rural ;
- relations uniques et triées ;
- belligérants et objectifs cohérents ;
- dates et ticks cohérents ;
- checksum et identité de sauvegarde.

Prévu aux étapes 4 et 5 :

- validation d'un data-pack complet et de ses registres ;
- règles de contenu (nations fictives, frontières irrégulières, pas de données réelles) ;
- validation croisée de tous les fichiers de données avant chargement.

Un pack qui échoue au niveau 1 ou 2 ne doit jamais être chargé partiellement : le runtime conserve l'état précédent et affiche une erreur exploitable.

## Fixtures de validation

`tests/schemas/valid/` contient 22 objets minimaux :

`config`, `culture`, `data-pack`, `diplomacy`, `event`, `game-data`, `government`, `ideology`, `index`, `localization`, `missile`, `names`, `nation`, `religion`, `resource`, `revolution`, `scenario`, `save`, `technology`, `unit`, `war`, `world`.

Elles sont volontairement petites. Le monde de test est une grille jitterée de 8 provinces ; il ne respecte pas les règles de production sur l'irrégularité. Il sert uniquement à prouver que les schémas et les `$ref` fonctionnent ensemble.
