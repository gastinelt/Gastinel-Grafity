# Plan d'action en 30 phases — GrafityGastinelAlpine (Gastinel Desktop)

> **Objectif final** : transformer le prototype actuel (~15–20 % d'achèvement, démo console Rust à 55 tests) en un **environnement de bureau complet, personnalisable, réactif et réellement utilisable** sur **Alpine Linux** (cible matérielle : **Samsung Galaxy Book5**, Intel Core Ultra 5 225U / Xe-LPG, 8 Go LPDDR5x 8733 MT/s), découpé pour être exécuté phase par phase par **Antigravity Cloud / Claude Code Cloud**.

---

## Règles d'architecture & Décisions validées

1. **Direction Artistique & Personnalisation** :
   - **Thème clair par défaut** (`ClairMinimaliste`), personnalisable ensuite par l'utilisateur (thèmes, couleurs d'accent, flou Kawase, échelle DPI, disposition des barres/raccourcis).
   - **Charte** : typographie `Inter`, icônes vectorielles Papicons (**0 emoji** dans l'UI).
2. **Politique dépendances (100 % hors-ligne pragmatique)** :
   - Compilation **100 % offline** conservée via `cargo vendor`, en autorisant de petites crates pure-Rust auditées (ex. `ab_glyph` / `ttf_parser`) + FFI direct sur les `.so` système Alpine (`libvulkan.so.1`, `libseat.so`, `libinput.so`, `libxkbcommon.so`).
3. **Contrainte `musl` + `dlopen` Vulkan** :
   - Sur `x86_64-unknown-linux-musl`, le linkage statique (`+crt-static`) casse `dlopen`. `.cargo/config.toml` doit forcer `-C target-feature=-crt-static` avec `vulkan-loader` installé sur Alpine.
4. **Stratégie affichage & Wayland** :
   - `VK_KHR_display` entre en conflit avec l'import `zwp_linux_dmabuf_v1` de clients Wayland externes (modifiers + sync fences + relâchement DRM master sur switch VT).
   - Le moteur repose donc sur un **rastériseur software + Mesa Lavapipe** pour les tests headless Cloud/CI, un backend **Win32 GDI** pour itérer vite sous Windows 11, puis **DRM/KMS atomique + `VK_EXT_image_drm_format_modifier`** sur Alpine.
5. **Cible noyau & pilotes Galaxy Book5 (`225U`)** :
   - Arrow Lake-U (`i915`/`xe`), Wi-Fi BE200/AX211 et audio DSP SOF exigent un **Kernel ≥ 6.5** (viser **Alpine Edge** ou dernière stable récente) + `linux-firmware-intel` + `mesa-vulkan-intel` + `sof-firmware` + `iwd` + `pipewire`.

---

## Consignes pour exécution par Agent Cloud (1 phase = 1 PR)

Avant de coder une phase `N` :
- Ne traiter **qu'une seule phase à la fois** (et uniquement ses critères d'acceptation).
- Ne jamais casser `cargo test --offline` ; toute nouvelle dépendance Rust doit être vendored dans `vendor/` (`offline = true`).
- Vérifier chaque étape visuelle/Vulkan dans le conteneur CI headless (**Mesa Lavapipe** / export PNG-PPM / `weston --backend=headless-backend.so` / **QEMU**).
- Ne jamais inclure d'informations personnelles, clés ou chemins absolus locaux dans le code ou les commits.

---

## Étape 1 — Assainissement, Toolchain musl & CI Cloud (Phases 1 à 5)

### Phase 1 — Tri du dépôt & préparation GitHub
- **Objectif** : sortir ou archiver les dossiers non trackés `Server/` (contenant notamment l'ISO `alpine-standard-3.21.7-x86_64.iso`) et `Desktop/`, nettoyer `.gitignore`, auditer les chemins/identifiants personnels avant tout push.
- **Viabilité** : ✅ Trivial. Indispensable avant d'ouvrir le repo aux agents Cloud.
- **Validation** : `git status --ignored` propre, workspace unique à la racine.

### Phase 2 — Toolchain `musl` dynamique & configuration `cargo vendor`
- **Objectif** : créer `.cargo/config.toml` avec `-C target-feature=-crt-static` pour `x86_64-unknown-linux-musl` ; configurer `vendor/` pour garder la compilation 100 % offline même avec des micro-crates pure-Rust.
- **Viabilité** : ✅ Indispensable (`+crt-static` empêche `dlopen("libvulkan.so.1")` sous musl).
- **Validation** : build musl dynamique réussi et vérifié (`ldd` / `readelf -d`).

### Phase 3 — Environnement CI Headless Alpine + Lavapipe
- **Objectif** : ajouter un `Dockerfile.ci` et un workflow CI basé sur Alpine avec `rust`, `cargo`, `vulkan-loader`, `mesa-vulkan-swrast` (Lavapipe), `weston`, `weston-backend-headless`.
- **Viabilité** : ✅ Éprouvé (Lavapipe exécute Vulkan sur CPU sans GPU physique en Cloud).
- **Validation** : `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.x86_64.json vulkaninfo --summary` voit `llvmpipe` et `cargo test --offline` passe en CI.

### Phase 4 — Architecture unifiée Backend (`DisplayBackend` & `InputEvent`)
- **Objectif** : définir des traits propres dans `gastinel-platform` séparant le moteur UI/compositeur des cibles : `HeadlessBackend` (CI), `Win32Backend` (dev Windows), `LinuxDrmBackend` (Alpine).
- **Viabilité** : ✅ Direct ; permet aux agents Cloud de tout tester en `HeadlessBackend`.
- **Validation** : tests d'intégration exécutant des frames complètes sur le backend headless.

### Phase 5 — Vraie boucle événementielle dans `gastinel-desktop`
- **Objectif** : remplacer le scénario linéaire `println!` de [`gastinel-desktop/src/main.rs`](../gastinel-desktop/src/main.rs) par une boucle temps réel (`EventLoop`), gérant `DamageRegion`, timers et flags CLI (`--setup-wizard`, `--headless-frames N`, `--backend=win32|drm|headless`).
- **Viabilité** : ✅ Direct.
- **Validation** : `gastinel-desktop --backend=headless --headless-frames 10` tourne, traite des événements simulés et quitte proprement (`exit 0`).

---

## Étape 2 — Rendu Réel : Typographie, Shaders & Pixels (Phases 6 à 10)

### Phase 6 — Vraie typographie `Inter` & icônes Papicons (0 emoji)
- **Objectif** : remplacer les métriques simulées de [`InterAtlas`](../crates/gastinel-core/src/font/mod.rs) par une vraie rastérisation d'`Inter` (Regular/Medium/SemiBold/Bold, accents français) via `ab_glyph`/`ttf_parser` vendored (ou atlas `.bin` pré-généré), et remplacer les emojis résiduels (`🖥️`, `📝`, `✨`) par des icônes vectorielles Papicons.
- **Viabilité** : ✅ Très propre en vendored/pré-rastérisé tout en gardant l'atlas ≤ 1–2 Mo.
- **Validation** : test CI générant une planche contact `inter_atlas_preview.ppm` lisible avec texte accentué.

### Phase 7 — Rastériseur Software de référence + Export PNG/PPM CI
- **Objectif** : implémenter un rastériseur CPU fidèle consommant `RenderQueue` (quads SDF arrondis, bordures, ombres, alpha blending, `GlyphInstance`) vers un framebuffer `RGBA8888`.
- **Viabilité** : ✅ Double usage : oracle visuel automatisé en Cloud (captures PPM/PNG en CI) + fallback universel.
- **Validation** : rendu du bureau clair + OOBE en images PPM/PNG générées automatiquement en CI.

### Phase 8 — Fenêtre interactive sur Windows (`Win32Window` + GDI)
- **Objectif** : brancher [`Win32Window`](../crates/gastinel-platform/src/win32.rs) sur le rastériseur de la Phase 7 via GDI (`StretchDIBits`), avec clics souris, molette, clavier et redimensionnement en direct sous Windows 11.
- **Viabilité** : ✅ 100 % natif Win32, testable sans VM.
- **Validation** : lancement local sur Windows 11 permettant de manipuler fenêtres, launcher et thèmes à la souris.

### Phase 9 — Pipeline Vulkan : SPIR-V offline, `VkDevice` & Command Buffers
- **Objectif** : compiler les shaders GLSL de [`shaders.rs`](../crates/gastinel-vulkan/src/shaders.rs) en `.spv` (`glslc`/`glslangValidator` en build offline + `include_bytes!`), puis étendre [`loader.rs`](../crates/gastinel-vulkan/src/loader.rs) avec `VkDevice`, `VkQueue`, `VkCommandPool`, `VkCommandBuffer`, `VkRenderPass`, `VkPipeline` et `VkFence`/`VkSemaphore`.
- **Viabilité** : ✅ Entièrement testable en Cloud sous **Lavapipe** en rendu offscreen (`VkImage` → `vkCmdCopyImageToBuffer` → PPM).
- **Validation** : test CI Lavapipe rendant un quad arrondi SDF + du texte Inter via Vulkan avec vérification des pixels générés.

### Phase 10 — Mémoire UMA réelle & Flou Dual Kawase GPU
- **Objectif** : brancher [`UmaHostBuffer`](../crates/gastinel-vulkan/src/uma.rs) sur `vkAllocateMemory` (`HOST_VISIBLE | HOST_COHERENT` + `vkMapMemory`) et exécuter les 4 passes de [`DualKawasePipeline`](../crates/gastinel-vulkan/src/blur.rs) sur GPU + scissoring `DamageRegion`.
- **Viabilité** : ✅ Testable sous Lavapipe en CI puis sur Intel Xe-LPG ANV.
- **Validation** : benchmark offscreen Lavapipe/ANV validant 4 passes Kawase + zéro allocation heap dans la boucle de frame.

---

## Étape 3 — Noyau Compositor Linux, Entrées & Wayland Réel (Phases 11 à 15)

### Phase 11 — Gestion de session `seatd` réelle (`libseat`)
- **Objectif** : remplacer le stub [`seatd.rs`](../crates/gastinel-platform/src/seatd.rs) par un binding FFI léger sur `libseat.so`, avec machine à états `enable_seat` / `disable_seat` pour gérer proprement le switch TTY (`Ctrl+Alt+Fn`).
- **Viabilité** : ⚠️ Critique : sans `disable_seat` propre lors d'un switch VT, le clavier/écran se fige. `libseat` sécurise le passage `SCM_RIGHTS`.
- **Validation** : test sous Alpine/QEMU ouvrant `/dev/dri/card0` sans privilèges `root`.

### Phase 12 — Sortie vidéo DRM/KMS réelle (Multi-résolution & Page-Flip)
- **Objectif** : remplacer les valeurs codées en dur de [`drm.rs`](../crates/gastinel-platform/src/drm.rs) par la lecture des connecteurs/CRTC/modes KMS (`drmModeGetResources`, `drmModeGetConnector`), la détection hotplug via `inotify` sur `/dev/dri`, et le `page-flip` VSync.
- **Viabilité** : ⚠️ Prévoir affichage KMS simple (dumb buffer / KMS atomic) puis scanout Vulkan (`VK_EXT_image_drm_format_modifier`).
- **Validation** : boot en VM QEMU (`virtio-gpu-pci`) affichant le bureau dans la vraie résolution détectée.

### Phase 13 — Entrées Clavier/Souris/Touchpad (`libinput` + `xkbcommon` AZERTY)
- **Objectif** : connecter `libinput` (via les FDs fournis par `seatd`) pour souris, clavier, défilement et gestes trackpad (tap-to-click, scroll 2 doigts), plus `libxkbcommon` pour le layout **AZERTY FR** exact (`@`, `€`, accents, touches mortes, `Super`/`Alt`).
- **Viabilité** : ✅ FFI direct vers `libinput.so` + `libxkbcommon.so` disponibles en paquets Alpine standards.
- **Validation** : saisie AZERTY complète, accents et curseur fluides en VM QEMU.

### Phase 14 — Serveur Wayland filaire réel (Clients SHM)
- **Objectif** : transformer [`WaylandServer`](../crates/gastinel-platform/src/wayland.rs) en vrai serveur socket Unix (`$XDG_RUNTIME_DIR/wayland-0`) parlant le protocole binaire `wl_display`, `wl_registry`, `wl_compositor`, `wl_shm`, `wl_surface`, `wl_seat`, `xdg_wm_base`, `xdg_toplevel`.
- **Viabilité** : ⚠️ Gros lot. Testable en CI Cloud headless contre `foot` ou un client Wayland de test.
- **Validation** : un terminal `foot` (ou client SHM) se connecte en CI, soumet ses buffers et apparaît dans le framebuffer exporté du compositeur.

### Phase 15 — Accélération clients GPU (`linux-dmabuf` & Sync Fences)
- **Objectif** : implémenter `zwp_linux_dmabuf_v1` avec négociation stricte des DRM format modifiers supportés (`vkGetPhysicalDeviceExternalImageFormatProperties`) et attente des sync fences explicites avant composition (cible : Firefox / Chromium fluides).
- **Viabilité** : ⚠️ **Phase la plus délicate** (modifiers Intel CCS, synchronisation). Conserver `wl_shm` en repli si un modifier est refusé.
- **Validation** : rendu navigateur fluide sous Alpine + ANV / QEMU Venus sans corruption d'image.

---

## Étape 4 — Personnalisation Complète & Expérience Bureau (Phases 16 à 20)

### Phase 16 — Configuration persistante & Rechargement à chaud
- **Objectif** : schéma `~/.config/gastinel/desktop.conf` (valeurs par défaut : thème clair `ClairMinimaliste`, police `Inter`, 0 emoji) avec validation et rechargement à chaud sans redémarrer la session.
- **Viabilité** : ✅ Direct.
- **Validation** : tests unitaires de parsing/sauvegarde + application immédiate à la frame suivante.

### Phase 17 — Personnalisation visuelle poussée
- **Objectif** : sélecteur de thèmes (Clair minimaliste par défaut, Sombres/OLED en option), couleurs d'accent sur mesure, rayons d'arrondis, intensité du flou Dual Kawase, facteur d'échelle DPI (`100 %` / `125 %` / `150 %`), fond d'écran procédural ou image locale.
- **Viabilité** : ✅ Naturel grâce aux shaders SDF paramétrés.
- **Validation** : persistance complète de tous les réglages visuels entre deux redémarrages.

### Phase 18 — Personnalisation du Workspace & Fenêtrage
- **Objectif** : rendre le layout modulable : **TopBar** ou **barre latérale style Arc**, dock optionnel, zones de *snap* / tiling automatique, bureaux virtuels 1–4, et **éditeur de raccourcis clavier** (`Super+Space`, `Super+1..4`, `Alt+Tab`).
- **Viabilité** : ✅ Direct (`workspace.rs` et `topbar.rs` déjà ébauchés).
- **Validation** : tests de drag/resize/snap, changement de barre et raccourcis personnalisés.

### Phase 19 — Shell interactif complet (Spotlight, Notifications, Menus)
- **Objectif** : brancher Spotlight (`Launcher` : indexation `.desktop`, calculatrice inline, recherche fichiers, bascule prompt IA `?`), `Alt+Tab` temps réel, menus contextuels contextuels et `NotificationCenter` branché sur IPC local.
- **Viabilité** : ✅ Direct.
- **Validation** : lancement d'applications au clavier depuis Spotlight + affichage/fermeture de notifications réelles.

### Phase 20 — Écran de verrouillage, Veille & Gestion Énergie Laptop
- **Objectif** : brancher `LockScreen` sur l'authentification utilisateur Alpine (`pam` / `shadow`), gérer la fermeture du capot (`acpid`), le verrouillage auto sur inactivité et la mise en veille (`zzz` / `mem`).
- **Viabilité** : ✅ Standard sur Alpine (`acpid`).
- **Validation** : verrouillage/déverrouillage réel testé en VM + hook pré-veille vérifié.

---

## Étape 5 — Applications Natives Réelles & IA Locale (Phases 21 à 25)

### Phase 21 — Terminal natif réel (`PTY` + parseur ANSI/VT100)
- **Objectif** : doter `TerminalApp` d'un vrai pseudo-terminal (`posix_openpt` + `/bin/sh`), gestion des séquences ANSI/256 couleurs, scrollback, copier-coller et onglets multiples.
- **Viabilité** : ✅ FFI libc minimal très léger.
- **Validation** : exécution interactive de `sh`, `htop`, `vi`, `apk` dans le terminal Gastinel.

### Phase 22 — Fichiers (`FilesApp`) & Éditeur (`Gastinel Write`) réels
- **Objectif** : `FilesApp` sur vrai système de fichiers (`std::fs`, corbeille `~/.local/share/Trash`, ouverture de fichiers). `Gastinel Write` : ouvrir/éditer/sauvegarder `.md`/`.txt`, sélection souris, presse-papiers Wayland (`wl_data_device`), annuler/refaire (`Ctrl+Z`).
- **Viabilité** : ✅ Rust `std::fs` pur.
- **Validation** : cycle complet lecture/édition/sauvegarde disque + copier-coller entre Write et Terminal.

### Phase 23 — Moniteur Système, Capture d'écran, Calculatrice & Horloge réels
- **Objectif** : `GastinelMonitor` lisant les vraies sondes Linux (`/proc/stat`, `/sys/class/drm`, `/sys/block/zram0/mm_stat`, batterie `/sys/class/power_supply/BAT*`) ; `ScreenshotTool` enregistrant le framebuffer réel en PNG/BMP ; `Calculator` ASM + `ClockApp` reliés à l'horloge système.
- **Viabilité** : ✅ Direct.
- **Validation** : tests unitaires des parseurs `/proc`/`/sys` + fichier capture non vide généré.

### Phase 24 — Paramètres Système & Gestionnaire `APK` réels
- **Objectif** : `SettingsApp` pilotant Wi-Fi (`iwd`/`iwctl`), audio (`wpctl`/PipeWire), luminosité (`/sys/class/backlight`), clavier et toute la **personnalisation** (Phases 16–18). `PackageManagerApp` exécutant `apk search` / `apk info` / `doas apk add|del|upgrade` en direct.
- **Viabilité** : ✅ Wrappers `Command::new` fiables sous Alpine.
- **Validation** : installation d'un paquet test et modification luminosité/Wi-Fi depuis l'UI en VM.

### Phase 25 — Assistant IA local `Gemma 4` (`llama-server` streaming)
- **Objectif** : relier `GemmaChatApp`, Spotlight (`?`) et `AiDashboardApp` en TCP/HTTP (`127.0.0.1:8080`, streaming SSE) à `llama-server` Vulkan, avec chargement/déchargement à la demande pour libérer la RAM sur 8 Go.
- **Viabilité** : ✅ Client HTTP/SSE minimal sur `std::net::TcpStream` sans dépendance lourde.
- **Validation** : test CI avec serveur mock HTTP SSE + test réel `Gemma-4-E2B` Vulkan.

---

## Étape 6 — Intégration Galaxy Book5, OOBE & Livraison (Phases 26 à 30)

### Phase 26 — Profil matériel Alpine Galaxy Book5 (`Core Ultra 5 225U`)
- **Objectif** : figer l'ensemble noyau/firmwares Alpine (Kernel ≥ 6.5, `linux-firmware-intel`, `mesa-vulkan-intel`, `intel-media-driver`, `sof-firmware`, `iwd`, `pipewire`, `wireplumber`, `seatd`, `acpid`), vérifier `xe` vs `i915` et l'audio SOF.
- **Viabilité** : ✅ Confirmé pour Arrow Lake-U ; à valider sur le PC cible (`dmesg`, `aplay -l`, `vulkaninfo`).
- **Validation** : checklist matérielle verte (GPU Vulkan ANV, Wi-Fi, son, trackpad, touches Fn).

### Phase 27 — Budget RAM (< 100 Mo au repos), `zram` & Autonomie batterie
- **Objectif** : profiler la consommation réelle (`smaps_rollup`) de la session complète hors IA, tuner `zram` LZ4 et OOM guard pour 8 Go LPDDR5x, et vérifier `0 %` GPU idle avec rendu événementiel.
- **Viabilité** : ✅ Atteignable pour le shell natif (~40–90 Mo) sans duplication inutile de buffers.
- **Validation** : rapport mesuré sur cible (`< 100 Mo` au repos hors modèle IA).

### Phase 28 — Assistant OOBE (`SetupWizard`) branché au système réel
- **Objectif** : connecter les 6 étapes de [`SetupWizard`](../crates/gastinel-shell/src/wizard.rs) au système : clavier, scan/connexion Wi-Fi réel (`iwctl`), création utilisateur + `doas`, sélection disque avec garde-fous stricts anti-écrasement dual-boot Windows 11 + mode `--dry-run`, création `/etc/gastinel/installed.lock`.
- **Viabilité** : ⚠️ Garde-fous critiques sur le partitionnement NVMe.
- **Validation** : installation complète testée dans **QEMU** sur disque virtuel (vierge + dual-boot simulé).

### Phase 29 — Usine à Image Bootable & Déploiement Live USB
- **Objectif** : moderniser [`build-alpine-rootfs.ps1`](../tools/build-alpine-rootfs.ps1) et [`deploy-native.sh`](../tools/deploy-native.sh) pour générer une image / clé USB live UEFI bootable d'Alpine + Gastinel Desktop, testée automatiquement dans **QEMU + OVMF**.
- **Viabilité** : ✅ Automatisable en CI Linux/QEMU.
- **Validation** : boot UEFI automatisé en QEMU jusqu'à l'OOBE graphique sans erreur OpenRC.

### Phase 30 — Recette Finale Galaxy Book5 & Release `v1.0`
- **Objectif** : validation live USB sur le Samsung Galaxy Book5 (dual-boot intact, personnalisation complète, Wayland Firefox/Foot, Gemma 4 Vulkan, mesures réelles RAM/FPS/autonomie), mise à jour finale de la documentation et tag `v1.0`.
- **Viabilité** : ✅ Validation finale intégrée.
- **Validation** : checklist matérielle 100 % cochée + release taguée.

---

## Suite envisagée après la `v1.0`
- **Créateur de clé USB bootable (outil dédié)** : petite application graphique multiplateforme (Windows / Linux) s'appuyant sur les scripts de la Phase 29 pour écrire proprement l'image live/installateur Gastinel Alpine sur clé USB en 1 clic (sélection lecteur USB sécurisée, vérification SHA-256).
