# Civilisations — étape 0 (`/civ ping`)

Ce dossier contient le **mod natif minimal** de l'étape 0. Ce n'est pas un
mod de contenu `.hmod` : c'est un bundle **`.hnmod`** compilé par le launcher
Horizon sur un commit précis.

## Contenu

| Fichier | Rôle |
|---|---|
| `horizon-native.json` | Manifeste du bundle : id, version, jeu, pin Horizon, pin horizon-mixins. |
| `mixins/civ/horizon-mixin.toml` | Déclaration du module et des deux hooks de commande. |
| `mixins/civ/runtime.rs` | Logique du mod, injectée comme `crate::game::civ`. |
| `mixins/civ/register_commands.rs` | Adaptateur du hook `register_commands`. |
| `mixins/civ/perform_command.rs` | Adaptateur du hook `perform_command`. |
| `package.py` | Fabrique `civ.hnmod` de façon reproductible. |

Le pin est celui d'Orbital Laser 0.5.14, vérifié avec le launcher 0.2.37+ :

- Horizon : `89f236e3ef8e8b554bef507216b86ca90b552213`
- horizon-mixins : `a2af5c0dd4117d51ecd70066ba7a00b4a2c10dab`

## Construire le fichier

```powershell
cd C:\Users\imano\Downloads\Horizon\civ-ping
python package.py
```

Cela produit `civ.hnmod` dans ce dossier. Le zip contient exactement
`horizon-native.json` et `mixins/civ/…` (uniquement `.rs` et `.toml`).

## Installer et compiler dans Horizon

1. Ferme Horizon.
2. Trouve le dossier de l'instance :

   ```powershell
   horizon-launcher instance list
   ```

3. Crée le dossier `horizon-native-mods` dans l'instance s'il n'existe pas,
   puis copie `civ.hnmod` dedans.
4. Vérifie que le mod est activé dans `horizon-mods.txt` de l'instance :

   ```text
   mod.civ:true
   ```

5. Compile le jeu avec le mod (le launcher télécharge son propre Rust) :

   ```powershell
   horizon-launcher native-build --instance "<nom ou chemin de l'instance>" --trust-native-code
   ```

   La première compilation prend plusieurs minutes (Rust + le jeu entier).
   Le launcher affiche une fenêtre de confiance : le code natif s'exécute avec
   les droits de l'utilisateur, donc ne compile que du code que tu as lu.

6. Lance l'instance. Dans un monde, tape :

   ```text
   /civ ping
   ```

   Réponse attendue : `Civ: pong`.

## Portée de l'étape 0

La commande est enregistrée côté serveur et traitée dans
`native_perform_command`. Elle fonctionne en solo (serveur intégré), en LAN
(chez l'hôte) et dans un export serveur dédié compilé avec
`--server-output`. Aucun contenu, aucune sauvegarde et aucune interface ne
sont ajoutés à ce stade.
