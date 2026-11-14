# CLAUDE.md — Gastinel Grafity

Notes de travail pour Claude. Répondre en **français**.

## Projet
Environnement de bureau ultra-léger en **Rust** pour **Alpine Linux** (sans `systemd` : OpenRC + `seatd`),
ciblant un portable 8 Go de RAM (GPU Intel intégré). Prototype ~15–20 %.
Feuille de route : `docs/PLAN_30_PHASES.md`. Dossier de reprise historique : `Conversation.md`.

## Règles absolues
- **100 % hors-ligne** : aucune dépendance réseau ni crate externe téléchargée. Rust pur + ABI Vulkan dynamique en C.
- **Zéro allocation dynamique dans la boucle de rendu** : utiliser `FrameArena` (réinitialisée par trame) et `Pool<T>` (générationnel, anti-ABA).
- **Rendu événementiel** : ne redessiner que les `DamageRegion`, 0 % d'activité GPU au repos.
- **Empreinte visée** : < 100 Mo de RAM au repos (hors modèle IA local).
- **DA** : thème clair par défaut, typographie `Inter`, icônes Papicons, **pas d'emojis dans l'interface**.

## Workspace
- `crates/gastinel-core` — FrameArena, Pool, géométrie, DamageRegion, RenderQueue, atlas typo
- `crates/gastinel-simd` — culling / maths SSE2-AVX2 (`asm!` x86_64)
- `crates/gastinel-vulkan` — loader Vulkan dynamique, tampons UMA, shaders SDF / Kawase
- `crates/gastinel-shell` — TopBar, Spotlight, Workspaces, OOBE, thèmes, apps, chat IA
- `crates/gastinel-platform` — Win32, seatd, DRM/KMS, Wayland (encore des ébauches)
- `gastinel-desktop` — binaire principal (démo console pour l'instant)
- `rootfs-overlay/`, `tools/` — fichiers système OpenRC et scripts de déploiement Alpine

## Commandes
```bash
cargo check --offline
cargo test --offline          # 55 tests
cargo run --bin gastinel-desktop --offline

# CI complète sous Alpine + Lavapipe + Weston headless (Phases 2-3)
docker build -f Dockerfile.ci -t gastinel-ci . && docker run --rm -v "$PWD":/src gastinel-ci
```

## Build musl / CI
- `.cargo/config.toml` : `offline = true`, sources crates.io remplacées par `vendor/`, `-crt-static` pour musl.
- Ne pas cross-compiler musl depuis un hôte glibc (ça lie `libc.so.6`) : construire sous Alpine.
- Dans ce sandbox cloud, Docker n'a pas de réseau direct : lancer `dockerd`, puis builder avec `--network host`,
  le proxy en `--build-arg https_proxy=...` et la CA `/root/.ccr/ca-bundle.crt` copiée en première couche
  (image dérivée dans le scratchpad, ne pas modifier `Dockerfile.ci`).
- Phases faites : 1, 2, 3.

## Tests (état au 2026-11-13 : 55/55 OK)
| Crate | Tests |
| :--- | ---: |
| gastinel-core | 6 |
| gastinel-platform | 3 |
| gastinel-shell | 37 |
| gastinel-simd | 5 |
| gastinel-vulkan | 4 |
| gastinel-desktop | 0 |

Mettre à jour ce tableau (et le README) quand des tests sont ajoutés.

## Prochaines étapes
1. Protocole Wayland XDG-Shell bas niveau (afficher de vraies apps : `foot`, etc.).
2. Entrées réelles via `libinput` / `evdev`.
3. Vulkan : `VkDevice`, swapchain, SPIR-V, passes GPU.
4. Valider le déploiement (QEMU / clé USB) avec `tools/`.
