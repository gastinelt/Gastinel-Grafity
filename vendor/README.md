# vendor/

Dépendances Rust externes copiées localement pour garder un build 100 % hors-ligne
(voir `.cargo/config.toml`). Le workspace n'en a aucune pour l'instant.

Ajouter une micro-crate pure-Rust auditée (ex. `ttf-parser`) :

```bash
# 1. Déclarer la dépendance dans le Cargo.toml du crate concerné
# 2. Sur une machine connectée, la copier ici :
cargo vendor --versioned-dirs vendor
# 3. Committer vendor/ ; les builds suivants restent hors-ligne
```
