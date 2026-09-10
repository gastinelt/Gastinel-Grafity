#!/bin/sh
# /tools/deploy-native.sh
# Script de déploiement direct de Gastinel Alpine sur un disque dur ou partition sans ISO
set -e

TARGET_DEV="$1"

if [ -z "$TARGET_DEV" ]; then
    echo "Usage: $0 /dev/sdX (ou /dev/nvme0n1pX)"
    echo "Exemple: $0 /dev/nvme0n1p3"
    exit 1
fi

echo "============================================================"
echo "   DÉPLOIEMENT NATIF DE GASTINEL ALPINE SUR $TARGET_DEV    "
echo "============================================================"

# 1. Formatage en ext4 ou btrfs optimisé
echo "[1/4] Formatage de la partition cible ($TARGET_DEV)..."
mkfs.ext4 -F -L "GASTINEL_OS" "$TARGET_DEV"

# 2. Montage temporaire
MOUNT_DIR="/mnt/gastinel-target"
mkdir -p "$MOUNT_DIR"
mount "$TARGET_DEV" "$MOUNT_DIR"

# 3. Extraction du rootfs Alpine
echo "[2/4] Extraction du rootfs Alpine Linux..."
if [ -f "alpine-minirootfs-*.tar.gz" ]; then
    tar -xzf alpine-minirootfs-*.tar.gz -C "$MOUNT_DIR"
else
    echo "Téléchargement du rootfs minimal..."
    wget -c https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/x86_64/alpine-minirootfs-3.21.3-x86_64.tar.gz
    tar -xzf alpine-minirootfs-3.21.3-x86_64.tar.gz -C "$MOUNT_DIR"
fi

# 4. Fusion de l'overlay système Gastinel
echo "[3/4] Injection de l'overlay système et d'OpenRC..."
cp -r ./rootfs-overlay/* "$MOUNT_DIR/"

# Activation des services OpenRC au boot
mkdir -p "$MOUNT_DIR/etc/runlevels/default"
mkdir -p "$MOUNT_DIR/etc/runlevels/boot"
mkdir -p "$MOUNT_DIR/etc/runlevels/sysinit"

ln -sf /etc/init.d/seatd "$MOUNT_DIR/etc/runlevels/default/seatd"
ln -sf /etc/init.d/gastinel-oobe "$MOUNT_DIR/etc/runlevels/default/gastinel-oobe"
ln -sf /etc/init.d/networking "$MOUNT_DIR/etc/runlevels/default/networking"

# 5. Copie du binaire Gastinel
echo "[4/4] Installation du binaire Gastinel Desktop..."
mkdir -p "$MOUNT_DIR/usr/bin"
cp ./target/release/gastinel-desktop "$MOUNT_DIR/usr/bin/gastinel-desktop"
chmod +x "$MOUNT_DIR/usr/bin/gastinel-desktop"
chmod +x "$MOUNT_DIR/usr/bin/gastinel-session"
chmod +x "$MOUNT_DIR/etc/init.d/gastinel"*

umount "$MOUNT_DIR"
echo "============================================================"
echo "   INSTALLATION TERMINÉE ! VOUS POUVEZ REDÉMARRER.          "
echo "============================================================"
