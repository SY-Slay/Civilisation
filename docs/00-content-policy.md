# 00 — Politique de contenu

## Principe

**Civilisations** est une fiction géopolitique. Tous les pays, peuples, dirigeants, frontières, partis, religions, crises et conflits représentés sont inventés. Le mod ne cherche pas à simuler un pays réel, une élection réelle, une guerre réelle ou une arme réelle.

## Règles impératives

1. **Nations fictives.** Aucun nom, drapeau, dirigeant, parti ou territoire réel. Les noms sont générés ou écrits pour la fiction.
2. **Frontières fictives.** Les cartes ne reprennent aucune frontière réelle. Les contours sont produits par du bruit cohérent, des reliefs, des rivières et des côtes, jamais par des rectangles ou des lignes droites arbitraires.
3. **Symboles originaux.** Les drapeaux et emblèmes sont dessinés pour le mod. Pas de reprise de symboles d'États, de mouvements politiques ou de groupes réels.
4. **Pas de données réelles.** Aucune liste de dirigeants, de populations, de bases militaires, de cibles ou de coordonnées réelles.
5. **Guerre et missiles abstraits.** Les missiles sont des objets de simulation. Le mod ne fournit aucune instruction de fabrication, de ciblage ou d'utilisation d'une arme réelle.
6. **IA déterministe.** L'« IA » est un ensemble de règles, de scores et de tirages pseudo-aléatoires reproductibles. Aucun modèle de langage, aucun appel réseau, aucune génération de texte à l'exécution.
7. **Licences libres.** Le code est sous MPL-2.0. Les données sont sous CC-BY-SA-4.0. Les assets doivent être originaux ou compatibles, avec attribution.
8. **Pas de contenu haineux.** Les idéologies fictives peuvent être autoritaires, démocratiques, technocratiques ou révolutionnaires, mais le mod ne fait pas l'apologie de la haine, de la violence réelle ou d'un groupe réel.

## Frontières irrégulières

Le schéma JSON ne peut pas prouver qu'une frontière est « naturelle ». La validation métier de `civ-core` devra vérifier :

- polygone simple, sans auto-intersection ;
- surface minimale et absence de trous non déclarés ;
- arêtes partagées entre provinces adjacentes, sans décalage ;
- nombre minimal de sommets sur les longues frontières ;
- ratio de rectitude maximal par segment ;
- cohérence avec le relief, les rivières et les côtes ;
- absence de lignes droites sur plus d'une distance configurable.

Un échec de validation bloque le chargement du scénario avec un message explicite. Les fixtures de `tests/schemas/valid/` utilisent des polygones simples uniquement pour valider la forme JSON ; **ce ne sont pas des cartes de production**.

## IA et déterminisme

- Même graine + même état initial + mêmes commandes = même partie.
- Les décisions sont triées par identifiant stable avant évaluation.
- Les collections itérées sont ordonnées (`BTreeMap` ou tri explicite).
- Chaque sous-système possède un flux RNG dérivé (`ai`, `war`, `events`, `missiles`) pour éviter qu'un changement local modifie tout le reste.
- Aucune décision ne dépend de l'horloge murale, du framerate ou de l'ordre d'un `HashMap`.

## Contenu tiers

Un pack tiers peut ajouter des nations, des cartes ou des événements s'il :

- déclare `civ` en dépendance ;
- utilise les mêmes schémas ;
- ne contient pas de contenu réel haineux ou diffamatoire ;
- fournit ses licences et attributions ;
- passe la validation métier avant chargement.
