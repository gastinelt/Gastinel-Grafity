# Guide de Performance & Multitâche : Intel Core Ultra 5 & 8 Go LPDDR5x (8733 MT/s)

Ce guide détaille les réglages concrets pour maximiser les performances de ton PC portable sous **Gastinel Desktop** lors de l'exécution simultanée d'un modèle d'IA 2B, d'une session de dev (Rust, C, Python) et de bureautique Web.

---

## 1. Pourquoi ce setup est unique

| Composant | Spécification | Atout sous Gastinel |
| :--- | :--- | :--- |
| **CPU** | Intel Core Ultra 5 225U (12 cœurs / 14 threads) | 2 P-Cores pour la latence + 8 E-Cores pour la compilation en fond |
| **GPU** | Intel Graphics (Xe-LPG) | Inférence IA Vulkan native en mémoire unifiée (UMA) |
| **RAM** | 8 Go LPDDR5x @ 8733 MT/s | Débit exceptionnel (>70 Go/s) = 60 à 80 tok/s sur modèles 2B |
| **OS** | Gastinel Alpine (musl libc + OpenRC) | **< 85 Mo de RAM au repos** (99% de RAM libre pour tes apps) |

---

## 2. Optimisation du Développement (Rust, C++, Web)

### A. Remplacer `ld` par le linker ultra-rapide `mold`
Le linker classique (`ld` ou `lld`) consomme beaucoup de RAM lors de l'édition de liens de gros binaires Rust. `mold` divise la consommation mémoire par 4 et accélère le link par 10 :

1. Installer `mold` :
   ```sh
   apk add mold
   ```
2. Configurer Cargo globalement (`~/.cargo/config.toml`) :
   ```toml
   [target.x86_64-unknown-linux-musl]
   linker = "clang"
   rustflags = ["-C", "link-arg=-fuse-ld=mold"]

   [build]
   # Limiter les jobs de build parallèles à 6 pour laisser des cœurs et de la RAM à l'IA et au navigateur
   jobs = 6
   ```

### B. Optimiser `rust-analyzer` (Helix / Neovim / VS Code)
Par défaut, `rust-analyzer` indexe agressivement toutes les dépendances en RAM :
```json
{
  "rust-analyzer.cachePriming.enable": false,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.check.command": "check"
}
```
*Gain : Consommation de rust-analyzer plafonnée à ~350 Mo au lieu de 1.8 Go.*

---

## 3. Optimisation de la Bureautique & Navigateur (Firefox)

1. Activer le déchargement automatique des onglets inactifs :
   - Ouvrir `about:config`
   - Définir `browser.tabs.unloadOnLowMemory` à `true`
   - Définir `browser.low_commit_space_threshold_mb` à `1024`
2. Forcer l'accélération matérielle Wayland/VA-API (Intel Xe) :
   ```sh
   export MOZ_ENABLE_WAYLAND=1
   export MOZ_DISABLE_RDD_SANDBOX=1
   ```
   *La lecture vidéo 4K sur YouTube consomme alors moins de 3% de CPU et 150 Mo de RAM.*

---

## 4. Inférence IA : Modèle Choisi (Gemma 4 E2B HauhauCS Aggressive)

Grâce à la bande passante LPDDR5x de 8733 MT/s et au moteur Vulkan unifié (UMA) :

- **Modèle sélectionné** : `Gemma-4-E2B-Uncensored-HauhauCS-Aggressive-Q2_K_P.gguf`
- **Fichier local** : `%USERPROFILE%\Downloads\Quick Share\Gemma-4-E2B-Uncensored-HauhauCS-Aggressive-Q2_K_P.gguf` (3.00 Go)
- **Variante alternative disponible** : `Q4_K_P` (3.45 Go, sur appareil mobile / Samsung)
- **Vitesse estimée** : ~65 à 75 tokens/seconde constants
- **Points forts** : Modèle non-censuré, ultra-optimisé pour les instructions directes, réactif sur le code et le français, aucune latence perceptible.
- **Budget VRAM UMA** : ~2.9 Go alloués dynamiquement dans la LPDDR5x, laissant **près de 5 Go de RAM physique libre** pour le multitâche et la compilation !

---

## 5. Surveillance en Direct (Commandes Utiles)

- **Vérifier l'état de la zram et le taux de compression :**
  ```sh
  zramctl
  ```
  *(Affiche le volume brut compressé et le gain mémoire réel)*
- **Lancer l'IA instantanément en CLI :**
  ```sh
  ask-ai
  ```
- **Surveiller l'usage CPU/RAM sans bloat :**
  ```sh
  htop
  ```
