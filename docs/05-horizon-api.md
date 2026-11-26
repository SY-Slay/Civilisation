# 05 — APIs Horizon vérifiées et limites

## Épinglage

Le projet cible :

- Horizon `26.3` ;
- commit `9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0` ;
- horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57` ;
- paire vérifiée par Orbital Laser `0.5.14`.

Le manifeste natif (`horizon-native.json`) doit contenir `schema`, `id`, `version`, `game`, `horizon`, `mixins`. Pour les versions actuelles du launcher, le bundle natif est un `.hmod` avec ce manifeste à la racine et ses sources sous `mixins/civ/`. Le `.hnmod` est l'ancien format.

## Hooks utilisés par l'architecture

| Hook / helper | API | Usage prévu | Limite vérifiée |
|---|---:|---|---|
| `native_register_commands` | 8 | Enregistrer `/civ ...` | Arbre de commandes par joueur ; les commandes sont reconstruites au tick suivant |
| `native_perform_command` | 8 | Recevoir une commande joueur | Le joueur est échangé dans l'état ; répondre avec `native_feedback` |
| `native_perform_console_command` | 99 | Commandes admin sur serveur dédié | Console seulement, niveau 4, pas de joueur ; la console du propriétaire solo n'est pas couverte |
| `native_server_tick` | 4 | Tick autoritaire de simulation | Début/fin de tick serveur non pause ; pas de joueur défini |
| `native_client_tick_pre/post` | 40 | Tick client (UI, caméra, alertes) | Côté client uniquement ; jamais autorité |
| `native_world_lifecycle` | 3 | Ouverture/fermeture de monde, chargement de sauvegarde | Événements autoritaires ; ne remplace pas une sauvegarde atomique |
| `native_world_dir()` | helper serveur | Répertoire de sauvegarde du monde | Côté serveur ; écrire avec `std::fs` |
| `native_instance_dir()` | 326 | Trouver `<instance>/config/civ/` | Dossier d'instance ; chemins à valider |
| `native_net_send_to_player` | 40 | Envoyer un snapshot/delta à un joueur | Taille de message bornée ; le client peut se déconnecter |
| `native_net_send_to_server` | 40 | Envoyer une intention client | Le serveur doit valider ; jamais confiance au client |
| `native_net_from_client` | 40 | Recevoir un message client | Tous les mods du serveur le voient ; canal namespacé |
| `native_net_from_server` | 40 | Recevoir un message serveur | Tous les mods du client le voient ; canal namespacé |
| `native_hud` | hook HUD | HUD, alertes, letterbox | Dessin 2D ; ne pas bloquer le thread de rendu |
| `native_world_drawn` | 58 | Frontières et marqueurs dans le monde | Après le rendu du monde ; géométrie à borner |
| `native_mod_screen_open` | 111 | Ouvrir l'écran Civilisations | Écran propre au mod ; un seul écran de mod ouvert ; Escape ferme |
| `native_screen_render_post` | 78 | Dessiner l'écran | Dessin manuel ; pas de layout déclaratif |
| `native_screen_mouse_button_pressed_pre` | 80 | Clics sur l'écran | Boutons GLFW ; peut annuler le clic |
| `native_screen_key_pressed_pre` | 104 | Clavier sur l'écran | Touches nommées ; peut annuler |
| `native_screen_char_typed_pre` | 104 | Saisie de texte | Caractères ; peut annuler |
| `native_camera_pose` | 402 | Caméra cinématique | Ne charge pas les chunks ; le joueur et la main restent ceux du joueur |
| `native_camera_shake` | 82 | Impacts de missiles, explosions | Demande par frame ; valeurs bornées |
| `native_render_hand` | 87 | Masquer la main en cinématique | Ne peut que masquer, pas remplacer |
| `native_feedback` | 8 | Réponse à une commande | Texte littéral ; pas de retour console hors commande |
| `native_tell` | 8 | Message à un joueur | Côté serveur ; joueur ciblé |
| `native_send_particles` | 11 | Particules d'impact | Particules vanilla supportées ; item/trail/vibration non supportés |
| `native_play_sound` | helper | Sons d'impact/alerte | Son côté client/serveur selon le helper ; vérifier la portée |

Les noms exacts des helpers de son et de particules doivent être confirmés dans le commit épinglé avant l'implémentation. Le tableau ci-dessus ne remplace pas une compilation de test.

## Ce qui est explicitement hors de l'architecture

### Pas de LLM

L'IA est une utility AI déterministe. Aucun appel réseau, aucun modèle de langage, aucune génération de texte à l'exécution.

### Pas de frappe Orbital Laser par coordonnées

Orbital Laser `0.5.14` permet de sélectionner et configurer un laser ; le tir passe par le Designator. Il n'expose pas de commande vérifiée « frappe à la position X/Z ». Le mod ne doit donc pas promettre :

- un ciblage orbital direct par `/civ missile` ;
- une intégration qui déclenche le laser d'Orbital sans API stable ;
- une synchronisation cachée avec ses internes.

L'intégration optionnelle se limite aux **effets visuels** si une API stable existe. Sinon, `ExplosionEffects` utilise les particules et sons de repli.

### Pas de chargement de chunks par la caméra

`native_camera_pose` (API 402) déplace la caméra, mais les chunks chargés restent ceux du joueur. Une cinématique qui traverse une carte non chargée montrera du vide ou du brouillard. Les plans doivent rester près du joueur, ou être précédés d'un chargement explicite.

### Pas d'UI déclarative

Les écrans personnalisés se dessinent et se manipulent à la main :

- `native_screen_render_post` pour le dessin ;
- hooks souris/clavier/caractère pour l'interaction ;
- `native_mod_screen_open` pour ouvrir l'écran.

Il n'existe pas de fichier de layout ou de composants déclaratifs vérifié pour ce besoin.

### Pas de contenu chargé par une API non vérifiée

Le chargement des données utilise :

- des données de référence embarquées à la compilation ;
- une matérialisation dans `<instance>/config/civ/` ;
- `std::fs` pour les overrides ;
- `native_instance_dir()` pour trouver le dossier.

Aucun hook de chargement de contenu Horizon n'est supposé. Si le launcher change l'emplacement de l'instance, le chemin est recalculé et validé.

### Pas de console riche

`native_run_command` s'exécute au niveau 4, sans sortie, et seulement hors d'un hook joueur. Il ne doit pas être utilisé pour des commandes de gameplay interactives. Les commandes passent par `native_register_commands` / `native_perform_command`.

## Build et installation

### Instance locale

1. construire le bundle `.hmod` avec `python tools/package_native.py` ;
2. accepter l'avertissement de code natif ;
3. copier le `.hmod` dans `horizon-native-mods/` de l'instance ;
4. lancer le jeu.

La commande exacte dépend du launcher installé (`native-build` avec `--trust-native-code`). L'étape 0 a utilisé ce flux et le documente dans `civ-ping/README.md`.

### Serveur dédié

Le build d'instance ne couvre pas le serveur. Il faut :

```text
horizon-launcher native-build <dossier> --server-output <sortie> --version <tag> --trust-native-code
```

Le commit épinglé doit correspondre. Un serveur dédié ne doit pas être mis à jour sans rebuild et test de sauvegarde.

## Sécurité

- Un mod natif n'est pas sandboxé : il a les droits du joueur.
- Le code natif doit être relu avant installation.
- Les données JSON ne doivent jamais être interprétées comme du code.
- Les messages réseau sont bornés, validés et appliqués uniquement côté serveur.
- Les chemins de fichiers sont normalisés et confinés au dossier d'instance ou de monde prévu.
- Aucun secret, token ou donnée personnelle dans les sauvegardes.

## Sources de vérité

- documentation native officielle de Horizon ;
- sources de `horizon-client` au commit épinglé ;
- catalogue `native-hooks-orbital.json` ;
- code et manifeste d'Orbital Laser `0.5.14` ;
- modèle de mod natif officiel.

Toute API non listée ici doit être vérifiée dans ces sources avant d'être utilisée. En cas de doute, l'architecture prévoit un repli ou une fonctionnalité explicitement non supportée.
