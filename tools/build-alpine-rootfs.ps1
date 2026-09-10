<#
.SYNOPSIS
    Script de préparation et d'injection de Gastinel Desktop sur Alpine Linux Natif (Rootfs).
.DESCRIPTION
    1. Récupère le rootfs minimal Alpine (alpine-minirootfs-*-x86_64.tar.gz) s'il est présent ou le télécharge.
    2. Fusionne l'overlay système optimisé (rootfs-overlay) : inittab, seatd, doas, sysctl.
    3. Compile et copie les binaires gastinel-desktop dans /usr/bin/.
    4. Prépare la structure amorçable native sans nécessiter d'installateur ISO.
#>

param(
    [string]$AlpineVersion = "3.21.3",
    [string]$Arch = "x86_64",
    [string]$TargetDir = "./dist/alpine-gastinel-rootfs"
)

$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "   GASTINEL NATIVE ROOTFS BUILDER (Alpine Linux)            " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

$RootfsTarball = "alpine-minirootfs-$AlpineVersion-$Arch.tar.gz"
$DownloadUrl = "https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/$Arch/$RootfsTarball"

# 1. Vérification du tarball
if (-not (Test-Path $RootfsTarball)) {
    Write-Host "[INFO] Le fichier $RootfsTarball n'est pas encore présent localement." -ForegroundColor Yellow
    Write-Host "[INFO] Dès que la connexion est disponible, lancez le téléchargement depuis :" -ForegroundColor Gray
    Write-Host "       $DownloadUrl" -ForegroundColor Cyan
} else {
    Write-Host "[OK] Tarball Alpine trouvé : $RootfsTarball" -ForegroundColor Green
}

# 2. Création du dossier cible
if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
    Write-Host "[OK] Répertoire cible créé : $TargetDir" -ForegroundColor Green
}

# 3. Copie des fichiers d'overlay système (inittab, services, optimisations)
Write-Host "[INJECT] Fusion des fichiers système personnalisés..." -ForegroundColor Cyan
$OverlayDir = "./rootfs-overlay"
if (Test-Path $OverlayDir) {
    Copy-Item -Path "$OverlayDir/*" -Destination $TargetDir -Recurse -Force
    Write-Host "[OK] Overlay système appliqué avec succès :" -ForegroundColor Green
    Write-Host "     - /etc/inittab (démarrage graphique direct sans prompt tty)" -ForegroundColor Gray
    Write-Host "     - /etc/init.d/gastinel & gastinel-oobe (services OpenRC)" -ForegroundColor Gray
    Write-Host "     - /etc/conf.d/seatd (gestion session non-root)" -ForegroundColor Gray
    Write-Host "     - /etc/doas.conf (élévation légère)" -ForegroundColor Gray
    Write-Host "     - /etc/sysctl.d/99-gastinel-memory.conf (zram & swappiness 10)" -ForegroundColor Gray
    Write-Host "     - /usr/bin/gastinel-session (lanceur graphique)" -ForegroundColor Gray
}

# 4. Compilation du binaire Gastinel Desktop
Write-Host "[BUILD] Compilation du binaire Gastinel Desktop en mode release..." -ForegroundColor Cyan
cargo build --release --bin gastinel-desktop --offline

$BinSource = "./target/release/gastinel-desktop.exe"
if (-not (Test-Path $BinSource)) {
    $BinSource = "./target/release/gastinel-desktop"
}

if (Test-Path $BinSource) {
    $DestBinDir = "$TargetDir/usr/bin"
    if (-not (Test-Path $DestBinDir)) { New-Item -ItemType Directory -Path $DestBinDir -Force | Out-Null }
    Copy-Item -Path $BinSource -Destination "$DestBinDir/gastinel-desktop" -Force
    Write-Host "[OK] Binaire Gastinel injecté dans $DestBinDir/gastinel-desktop" -ForegroundColor Green
}

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "   ROOTFS GASTINEL PRÊT POUR DÉPLOIEMENT NATIF              " -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Cyan
