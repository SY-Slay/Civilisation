# 04 — Commandes `/civ`

> **Statut :** l'analyseur et l'exécution testable sont implémentés dans `civ-horizon` (étape 3). L'enregistrement natif auprès de Horizon, les permissions serveur et l'affichage HUD seront branchés dans `glue.rs` à l'étape 4. Aucun nom d'API Horizon n'est supposé ici.

## Syntaxe

Une commande accepte le préfixe `/civ` ou `civ` :

```text
/civ status
civ status
```

Les identifiants sont sensibles à la casse et suivent `^[a-z][a-z0-9_]{2,63}$`. Les arguments sont séparés par des espaces. Une commande vide, un argument manquant, un argument en trop ou un identifiant invalide produit une erreur explicite ; la simulation n'est pas modifiée.

## Commandes disponibles

| Commande | Effet | Notes |
|---|---|---|
| `/civ ping` | Vérifie le pont et le déterminisme. | Réponse stable, sans effet de jeu. |
| `/civ status` | Affiche la date, le tick, les compteurs et la nation joueuse. | Lecture seule. |
| `/civ tick <n>` | Avance la simulation de `n` ticks. | `n` est un entier positif ou nul, plafonné à 10 000 000. Un jour fictif est exécuté tous les `ticks_per_day` ticks. |
| `/civ save [nom]` | Scelle une sauvegarde et la transmet à l'hôte. | Le nom par défaut est `civilisations`. L'écriture disque réelle appartient à `Host::request_save` (étape 4). |
| `/civ player <nation>` | Change la nation du joueur. | La nation doit exister. |
| `/civ nations` | Liste les nations triées par identifiant. | Affiche provinces, stabilité et PIB. |
| `/civ nation <nation>` | Fiche détaillée d'une nation. | Capitale, population, économie, armée, recherche, guerres, révolutions. |
| `/civ diplo <a> <b>` | Affiche la relation bilatérale. | Opinion, confiance, peur, tension, statut, ambassade. |
| `/civ war <a> <b> [province]` | Déclare une guerre. | La province cible est optionnelle ; sans cible, la première province frontalière déterministe est utilisée. Les traités bloquants sont refusés. |
| `/civ peace <guerre>` | Signe la paix d'une guerre active. | Le vainqueur est désigné par le score de guerre ; les provinces contrôlées par le vainqueur peuvent être transférées. |
| `/civ treaty <type> <a> <b>` | Signe un traité entre deux nations. | Types : `peace`, `alliance`, `non_aggression`/`nap`, `trade`/`commerce`, `defensive`, `ceasefire`, `customs_union`, `research`. |
| `/civ missile <nation> <type> <province>` | Lance un missile depuis la capitale. | Le type doit exister, être en stock, et la cible doit être à portée. L'interception est résolue par le RNG de la partie. |
| `/civ research <nation> <technologie>` | Démarre une recherche. | La technologie doit exister et ses prérequis doivent être satisfaits. |
| `/civ event <choix>` | Résout l'événement actif de la nation joueuse. | Le choix doit exister dans l'événement actif. Les effets différés sont persistés. |
| `/civ revolution <nation>` | Force une révolution. | Commande de test/admin ; elle ne remplace pas les conditions naturelles de déclenchement. |
| `/civ help` | Affiche l'aide. | Lecture seule. |

## Exemples

```text
/civ nations
/civ nation ardenne
/civ diplo ardenne kesh
/civ war ardenne kesh prov_0_2
/civ peace war_001
/civ treaty trade ardenne kesh
/civ missile kesh missile_short prov_0_0
/civ research ardenne tech_administration
/civ event accept
/civ revolution ardenne
/civ tick 2400
```

## Permissions et autorité

L'étape 3 ne fixe pas encore les permissions Horizon. La règle cible est :

- toute commande est autorisée côté serveur selon le niveau de permission de l'émetteur ;
- le client ne modifie jamais directement la sauvegarde ;
- les commandes de test (`tick`, `revolution`) sont réservées aux administrateurs ou au mode solo ;
- `save` demande l'écriture à l'hôte ; le serveur reste seul écrivain.

Le branchement des permissions et des commandes natives appartient à l'étape 4 et devra utiliser uniquement des APIs vérifiées dans le commit Horizon épinglé.

## Déterminisme

- Une commande est appliquée dans l'ordre de réception.
- Les identifiants sont stables et les collections sont triées avant itération.
- Les effets de commande utilisent le RNG sérialisé de la partie.
- Une commande invalide ne consomme ni tick, ni tirage aléatoire, ni effet différé.
- `tick` exécute les systèmes dans l'ordre fixe : économie, événements, IA, guerre, révolutions, missiles, réindexation.
