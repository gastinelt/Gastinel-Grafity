# Architecture du Système Natif Alpine + Gastinel OOBE

Ce document détaille la configuration système Alpine native modifiée pour offrir une simplicité d'installation comparable à Windows, avec un démarrage direct sur l'interface graphique de configuration (**Gastinel Setup Wizard / OOBE**).

---

## 1. Vue d'Ensemble : Le Démarrage Zéro-Ligne-de-Commande

Dans une installation standard d'Alpine Linux :
1. Démarrage en mode texte `localhost login:`.
2. Lancement manuel de scripts shell comme `setup-alpine`.
3. Configuration manuelle des interfaces, disques, mot de passe, etc.

Dans **Gastinel Alpine Natif** :
1. Le noyau Linux charge le rootfs minimal optimisé.
2. OpenRC initialise `udev` et `seatd` en arrière-plan (< 1.5 seconde).
3. **`inittab`** et le service **`gastinel-setup`** lancent immédiatement l'interface graphique Vulkan sur l'écran (`/dev/dri/card0`) sans passer par aucun terminal texte.
4. L'utilisateur est accueilli par l'assistant graphique **Gastinel Setup** (OOBE) :
   - Choix de la langue et disposition clavier (AZERTY / QWERTY).
   - Connexion Wi-Fi avec saisie de mot de passe graphique.
   - Création du compte utilisateur avec mot de passe.
   - Choix du disque cible ou validation de la partition.
   - Application des réglages système et passage immédiat au bureau Gastinel Desktop !

---

## 2. Arborescence du Système Modifié (`rootfs-overlay/`)

```text
rootfs-overlay/
├── etc/
│   ├── inittab                          # Démarrage automatique sans prompt tty texte
│   ├── init.d/
│   │   ├── gastinel                     # Service OpenRC du bureau
│   │   └── gastinel-oobe                # Service du Setup initial au premier boot
│   ├── conf.d/
│   │   └── seatd                        # Droits d'accès GPU et entrées pour non-root
│   ├── doas.conf                        # Élévation de privilèges sans bloat sudo
│   ├── apk/
│   │   └── repositories                 # Dépôts Alpine main & community activés
│   ├── sysctl.d/
│   │   └── 99-gastinel-memory.conf      # Optimisation RAM pour 8 Go (zram, swappiness=10)
│   └── motd                             # Message d'accueil épuré
└── usr/
    └── share/
        └── gastinel/                    # Assets et thèmes de l'assistant
```

---

## 3. Les Optimisations Système Alpine Spécifiques

- **`vm.swappiness = 10`** : Réduit le swapping agressif, réserve la mémoire aux applications.
- **`zram` (lz4)** : Compresse la RAM à la volée, offrant l'équivalent de 12 à 14 Go d'espace sans disque mécanique.
- **`seatd`** : Permet à l'utilisateur de piloter Vulkan et le matériel sans jamais être connecté en `root`.
