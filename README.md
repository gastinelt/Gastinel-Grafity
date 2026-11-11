# Gastinel Grafity

![Statut](https://img.shields.io/badge/statut-prototype%20(~15--20%25)-orange)
![Langage](https://img.shields.io/badge/Rust-2021-blue)
![Cible](https://img.shields.io/badge/OS-Alpine%20Linux-0D597F)
![Licence](https://img.shields.io/badge/licence-GPLv3-green)

> **README in English:** [Click here](#english) | **README en français :** [Cliquez ici](#francais)

---

### Résumé rapide (FR)

**Gastinel Grafity** *(nom provisoire)* est un environnement graphique ultra-léger, modulaire et personnalisable en **Rust**, pensé pour **Alpine Linux** (sans `systemd`) sur des machines à 8 Go de RAM (type ultrabook Intel Core Ultra). L'objectif est d'offrir un bureau réactif en thème clair par défaut, piloté par événements (`DamageRegion`), rendu via Vulkan / DRM-KMS et accompagné d'applications système natives et d'un assistant IA local optionnel.

> **État actuel :** le projet est encore au stade de **prototype architectural (~15–20 %, non finalisé)**. Les structures cœur (mémoire par arènes, SIMD, états UI des composants/apps, 55 tests unitaires) compilent 100 % hors-ligne, mais `gastinel-desktop` exécute pour l'instant une démonstration console sans sortie vidéo réelle. Le plan complet jusqu'à la `v1.0` est détaillé dans [`docs/PLAN_30_PHASES.md`](docs/PLAN_30_PHASES.md).

---

<a id="francais"></a>
## 🇫🇷 Documentation — Français

### 🎯 Vision & Objectifs

- **Empreinte mémoire visée** : rester sous **< 100 Mo de RAM au repos** (hors modèle IA local) pour laisser la quasi-totalité de la mémoire vive aux applications lourdes.
- **Rendu événementiel à la demande** : ne redessiner que les zones modifiées (`DamageRegion`) afin d'éviter toute activité GPU inutile au repos.
- **Direction Artistique & Personnalisation** : thème clair minimaliste par défaut, typographie `Inter`, icônes vectorielles Papicons (sans emojis dans l'interface), et personnalisation complète prévue (thèmes, couleurs d'accent, flou Dual Kawase, disposition des barres et raccourcis).
- **Écosystème Alpine natif** : intégration prévue avec `seatd` (sans `systemd`), `DRM/KMS`, `libinput`/`xkbcommon`, Wayland, `zram`, et un assistant de premier démarrage (OOBE).
- **Build 100 % reproductible hors-ligne** : compilation Rust workspace sans réseau requis (`cargo test --offline`).

---

### 🚧 État actuel (Prototype — pas encore utilisable au quotidien)

| Couche | État réel | Détails |
| :--- | :---: | :--- |
| `gastinel-core` & `gastinel-simd` | ✅ / ⚠️ | Arènes mémoire (`FrameArena`), pools générationnels, `DamageRegion`, `RenderQueue`, opérations SSE2 `asm!`. La rastérisation réelle `.ttf` d'`Inter` reste à brancher. |
| `gastinel-shell` | ⚠️ | États, layouts et génération de primitives pour TopBar, Spotlight, Workspaces, OOBE, 10 apps et chat IA codés en mémoire (37 tests), sans boucle interactive complète. |
| `gastinel-vulkan` | ⚠️ / ❌ | Chargement dynamique `vulkan-1.dll` / `libvulkan.so.1` et énumération des GPU OK ; `VkDevice`, swapchain, SPIR-V et flou GPU restent à implémenter. |
| `gastinel-platform` & `gastinel-desktop` | ❌ | `main.rs` est une démo console scénarisée. Les backends Win32, `seatd`, DRM/KMS, `libinput` et le serveur Wayland filaire sont à l'état d'ébauches/mocks. |
| `rootfs-overlay/` & `tools/` | ⚠️ | Scripts de déploiement et configuration OpenRC rédigés, en attente de validation automatisée QEMU / matériel réel. |

👉 **Feuille de route complète (30 phases)** : voir [`docs/PLAN_30_PHASES.md`](docs/PLAN_30_PHASES.md).

---

### 📁 Architecture du Workspace

```text
├── crates/
│   ├── gastinel-core/       # FrameArena, Pool<T>, géométrie, DamageRegion, RenderQueue, atlas typo
│   ├── gastinel-simd/       # Opérations vectorisées et calcul SSE2 en assembleur x86_64
│   ├── gastinel-vulkan/     # Loader Vulkan dynamique C-ABI, tampons UMA, shaders GLSL SDF / Kawase
│   ├── gastinel-shell/      # Shell UI (TopBar, Launcher, Workspaces, OOBE, Thèmes, Apps)
│   └── gastinel-platform/   # Couche plateforme (Win32, seatd, DRM/KMS, serveur Wayland)
├── gastinel-desktop/        # Binaire principal (actuellement démo console)
├── rootfs-overlay/          # Fichiers système et services OpenRC pour Alpine Linux
├── tools/                   # Scripts de préparation rootfs / déploiement / pile IA
├── docs/                    # Architecture et plan d'action en 30 phases
├── Cargo.toml               # Workspace Cargo (GPL-3.0-only)
└── LICENSE                  # Texte complet GNU GPL v3
```

---

### 🛠️ Compilation & Vérification (Hors-ligne)

> Prérequis : chaîne Rust (`rustc` / `cargo`) installée localement. Sous Alpine Linux `musl`, le chargement dynamique de `libvulkan.so.1` nécessite `-C target-feature=-crt-static` (prévu en Phase 2 du plan).

```bash
# Vérifier la compilation complète hors-ligne
cargo check --offline

# Lancer les 55 tests unitaires actuels
cargo test --offline

# Exécuter la démo console actuelle
cargo run --bin gastinel-desktop --offline
```

---

### 📄 Licence (FR)

Ce projet est distribué sous licence **GNU General Public License v3.0 (`GPL-3.0-only`)** — voir le fichier [`LICENSE`](LICENSE).

---

<a id="english"></a>
## 🇬🇧 Documentation — English

> **[Back to top / Retour en haut](#gastinel-grafity)**

### 🎯 Overview & Goals

**Gastinel Grafity** *(working title)* is an experimental, lightweight, event-driven desktop environment written in **Rust** for **Alpine Linux** (no `systemd`), targeting resource-constrained laptops (8 GB RAM). The goal is a customizable, responsive desktop with a light theme by default, on-demand damage-based rendering, native system apps, and optional local AI integration.

> **Current Status:** This repository is an **early architectural prototype (~15–20% complete, not a finished v1.0 desktop)**. Core data structures, SIMD helpers, UI component logic, and 55 unit tests build and pass offline, while `gastinel-desktop` currently runs a scripted console demonstration without graphical screen output yet. See [`docs/PLAN_30_PHASES.md`](docs/PLAN_30_PHASES.md) (in French) for the full 30-phase roadmap toward a working system.

---

### 🚧 Current Implementation Status

| Layer | Status | Notes |
| :--- | :---: | :--- |
| `gastinel-core` & `gastinel-simd` | ✅ / ⚠️ | Linear `FrameArena`, generational pools, `DamageRegion`, `RenderQueue`, `x86_64` SSE2 `asm!` ops. Real TTF font rasterization for `Inter` is not wired yet. |
| `gastinel-shell` | ⚠️ | In-memory UI state, layout, and quad/glyph generation for TopBar, Spotlight, Workspaces, OOBE wizard, 10 apps, and AI chat (37 tests); not wired to a live input/event loop yet. |
| `gastinel-vulkan` | ⚠️ / ❌ | Dynamic loading of `vulkan-1.dll` / `libvulkan.so.1` and physical device enumeration work; `VkDevice`, swapchain, SPIR-V compilation, and GPU passes are pending. |
| `gastinel-platform` & `gastinel-desktop` | ❌ | `main.rs` is a scripted CLI walkthrough. Win32 windowing, `seatd` IPC, DRM/KMS atomic modesetting, `libinput`, and wire-protocol Wayland are stubs/mocks. |
| `rootfs-overlay/` & `tools/` | ⚠️ | OpenRC init scripts and Alpine deployment helpers drafted, pending automated QEMU / bare-metal validation. |

---

### 📁 Workspace Layout

```text
├── crates/
│   ├── gastinel-core/       # FrameArena, Pool<T>, geometry, DamageRegion, RenderQueue, font atlas
│   ├── gastinel-simd/       # Vectorized culling and x86_64 SSE2 inline assembly math
│   ├── gastinel-vulkan/     # Dynamic C-ABI Vulkan loader, UMA buffers, GLSL SDF / Kawase shaders
│   ├── gastinel-shell/      # Desktop shell components, themes, OOBE wizard, system apps
│   └── gastinel-platform/   # Platform backends (Win32, seatd, DRM/KMS, Wayland compositor)
├── gastinel-desktop/        # Main binary (currently scripted console demo)
├── rootfs-overlay/          # Alpine Linux OpenRC & system overlay files
├── tools/                   # Rootfs assembly, deployment, and local AI setup scripts
├── docs/                    # Architecture notes and 30-phase action plan
├── Cargo.toml               # Cargo workspace (GPL-3.0-only)
└── LICENSE                  # Full GNU GPL v3 text
```

---

### 🛠️ Build & Test (Offline)

```bash
# Type-check all workspace crates offline
cargo check --offline

# Run the current 55 unit tests
cargo test --offline

# Run the current scripted console demo
cargo run --bin gastinel-desktop --offline
```

---

### 📄 License (EN)

Distributed under the **GNU General Public License v3.0 (`GPL-3.0-only`)**. See [`LICENSE`](LICENSE) for the full text.
