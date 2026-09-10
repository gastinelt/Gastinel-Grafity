# Gastinel Desktop (GGA)

> **Environnement graphique ultra-léger et événementiel en Rust & Vulkan, conçu pour Alpine Linux.**

Gastinel est un bureau moderne pensé pour minimiser l'empreinte mémoire (< 100 Mo de RAM au repos) et la consommation d'énergie, laissant ainsi un maximum de RAM (~8 Go) disponible pour les applications lourdes.

---

## 🚀 Caractéristiques Clés

- **Zéro dépendance réseau externe** : Écrit en Rust bas niveau sans couches intermédiaires lourdes.
- **Rendu Vulkan On-Demand & Damage-Driven** : Le GPU ne tourne **jamais dans le vide**. Aucun frame n'est soumis si aucune surface ou composant d'UI n'a changé (0% d'activité GPU au repos).
- **Damage Regions & Coalescing** : Seules les zones d'écran modifiées (*dirty rects*) sont redessinées et découpées via `vkCmdSetScissor`.
- **Gestion Mémoire par Arènes (Frame Arena)** : Les allocations temporaires par trame sont résolues via une arène linéaire préallouée (réinitialisation instantanée à zéro, aucun `malloc`/`free` récurrent dans la boucle critique).
- **Pools Générationnels** : Gestion sécurisée des fenêtres et surfaces via des identifiants avec compteur de génération (protection O(1) contre le problème ABA).
- **Accélération SIMD Hot Paths** : Vectorisation AVX2 / SSE2 pour les opérations critiques de découpe et de mélange alpha.
- **Shell UI Embarqué** : TopBar, lanceur d'applications type Spotlight avec recherche floue (*fuzzy match*), gestionnaire de bureaux virtuels et fenêtres intégrés directement dans le même processus.
- **Prêt pour Alpine Linux** : Conçu pour s'intégrer avec **`seatd`** (gestion de session sans `systemd`/`elogind`) et le noyau **DRM/KMS**.

---

## 📁 Architecture Modulaire

```text
├── crates/
│   ├── gastinel-core/       # Arènes mémoire, pools d'objets, géométrie, DamageRegion, EventQueue
│   ├── gastinel-simd/       # Optimisations SIMD (AVX2/SSE2) pour la mémoire et les pixels
│   ├── gastinel-vulkan/     # Chargeur dynamique Vulkan (zero-binding C-ABI), coordinateur on-demand
│   ├── gastinel-shell/      # TopBar, Launcher Spotlight, Workspaces, Window Manager
│   └── gastinel-platform/   # Abstraction Alpine Linux (seatd IPC, DRM/KMS outputs)
├── gastinel-desktop/        # Binaire principal du bureau
└── Cargo.toml               # Workspace Cargo
```

---

## 🛠️ Compilation & Exécution

### Compilation hors-ligne (100% offline) :
```bash
cargo check --offline
```

### Exécution des tests unitaires :
```bash
cargo test --offline
```

### Lancer le moteur :
```bash
cargo run --bin gastinel-desktop --offline
```

---

## 🐧 Déploiement sur Alpine Linux

Sur une installation fraîche d'Alpine Linux :
1. Activer le dépôt `community` et installer les paquets de base :
   ```sh
   apk add seatd libinput mesa-vulkan-intel vulkan-loader
   rc-update add seatd
   rc-service seatd start
   ```
2. Ajouter ton utilisateur au groupe `seat` et `video` :
   ```sh
   adduser $USER seat
   adduser $USER video
   ```
3. Compiler en mode release :
   ```sh
   cargo build --release
   ./target/release/gastinel-desktop
   ```
