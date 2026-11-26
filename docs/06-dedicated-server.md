# 06 — Serveur dédié : état et limites

## État actuel

Le fonctionnement sur un serveur dédié Horizon n'a pas été vérifié. Le bundle natif a été lancé et la simulation a été initialisée dans un monde Horizon existant, mais cela ne valide pas un serveur headless.

L'option `Civilisations.exe --server` n'est pas un serveur Horizon : elle démarre un simulateur console local en mémoire, sans écoute réseau, sans hooks Horizon et sans sauvegarde persistante. Elle sert uniquement à essayer les commandes et la simulation en ligne de commande.

Le code natif contient des hooks de tick serveur, de cycle de vie du monde, de commande console et de réseau. Leur présence dans le code ne constitue pas une preuve de compatibilité avec un serveur dédié. Aucune configuration serveur dédiée, intégration RCON ni autosauvegarde périodique n'est actuellement documentée comme implémentée.

## Comportements présents dans le code

- À l'ouverture d'un monde, le hook charge `autosave.json` ou `civilisations.json` depuis le répertoire de sauvegarde Civilisations; s'il n'en trouve aucun, il crée une session depuis le scénario embarqué.
- La simulation avance d'un tick Civilisations à chaque tick serveur actif.
- À la fermeture normale du monde, le hook tente d'écrire une sauvegarde `autosave`.
- Le code contient des hooks d'envoi et de réception réseau; le comportement sur un serveur dédié reste à tester.
- Le launcher propose `--sim` pour le simulateur interactif et conserve `--server` comme alias historique explicitement décrit comme simulateur console.

Ces comportements décrivent l'implémentation actuelle, pas une garantie de fonctionnement sur toutes les distributions ou configurations Horizon.

## Commandes natives actuellement enregistrées

Les usages ci-dessous correspondent aux déclarations du code `civ-horizon`. Leur autorisation et leur comportement côté serveur dédié ne sont pas validés séparément.

| Usage | Fonction |
|---|---|
| `/civ ping` | Vérifie la présence du mod |
| `/civ status` | Affiche l'état de la session |
| `/civ tick <ticks>` | Avance la simulation du nombre de ticks indiqué |
| `/civ save [name]` | Sauvegarde la session |
| `/civ player <nation>` | Associe le joueur à une nation |
| `/civ nations` | Liste les nations |
| `/civ nation <nation>` | Affiche une nation |
| `/civ diplo <a> <b>` | Affiche les relations diplomatiques |
| `/civ war <a> <b> [province]` | Déclare une guerre |
| `/civ peace <war>` | Met fin à une guerre |
| `/civ treaty <kind> <a> <b>` | Crée un traité |
| `/civ missile <nation> <type> <province>` | Lance une action missile |
| `/civ research <nation> <tech>` | Débloque une technologie |
| `/civ event <choice>` | Applique un choix d'événement |
| `/civ revolution <nation>` | Déclenche une révolution |
| `/civ help` | Affiche l'aide |

Les arguments entre chevrons sont obligatoires et ceux entre crochets sont facultatifs. Dans les déclarations natives Horizon, ils portent aussi un type explicite, par exemple `<ticks:integer>`.

## Travail nécessaire avant d'annoncer le support dédié

1. Construire le bundle avec le toolchain et la version Horizon ciblés.
2. Le charger sur un serveur dédié headless et vérifier son démarrage, les ticks, les commandes console et les paquets réseau avec au moins un client.
3. Vérifier la sauvegarde et la reprise après arrêt normal, puis tester le comportement en cas d'échec de sauvegarde.
4. Documenter uniquement les options de configuration réellement implémentées et les résultats mesurés.

Ne pas utiliser ce document comme procédure d'installation d'un serveur de production avant que ces validations aient été effectuées. N'effectuer aucun test sur un monde ou une sauvegarde utilisateur sans autorisation explicite.
