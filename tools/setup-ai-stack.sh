#!/bin/sh
# Gastinel AI Setup Script for Intel Core Ultra 5 225U (8GB LPDDR5x 8733 MT/s)
# Compiles llama.cpp with Vulkan GPU offloading and downloads optimized 2B models.

set -e

echo "============================================================"
echo "   GASTINEL AI POWERHOUSE SETUP (Intel Core Ultra 5 225U)   "
echo "   Mémoire : 8 Go LPDDR5x (8733 MT/s) | Moteur : Vulkan     "
echo "============================================================"

AI_DIR="/opt/gastinel-ai"
MODELS_DIR="$AI_DIR/models"
BIN_DIR="/usr/local/bin"

mkdir -p "$MODELS_DIR"
cd "$AI_DIR"

echo "[1/4] Installation des dépendances de compilation Alpine..."
apk add --no-cache build-base cmake git vulkan-loader-dev mesa-vulkan-intel linux-headers curl

echo "[2/4] Récupération et compilation de llama.cpp avec backend Vulkan..."
if [ ! -d "llama.cpp" ]; then
    git clone --depth 1 https://github.com/ggerganov/llama.cpp.git
fi

cd llama.cpp
mkdir -p build && cd build

# Compilation optimisée : Vulkan GPU offloading + AVX2 pour le Core Ultra 5
cmake .. \
    -DGGML_VULKAN=ON \
    -DGGML_NATIVE=ON \
    -DCMAKE_BUILD_TYPE=Release

cmake --build . --config Release -j$(nproc) --target llama-cli llama-server

cp bin/llama-cli "$BIN_DIR/llama-cli"
cp bin/llama-server "$BIN_DIR/llama-server"

echo "[3/4] Configuration du modèle IA 2B ultra-optimisé (Gemma 4 E2B)..."
cd "$MODELS_DIR"

# Modèle fourni localement par l'utilisateur : Gemma-4-E2B-Uncensored-HauhauCS-Aggressive
GEMMA_MODEL="Gemma-4-E2B-Uncensored-HauhauCS-Aggressive-Q2_K_P.gguf"

if [ ! -f "$GEMMA_MODEL" ]; then
    echo "Recherche du modèle Gemma 4 dans les dossiers locaux..."
    # 1. Vérification dans le dossier Téléchargements / Quick Share Windows (si monté)
    for candidate in \
        "/mnt/c/Users/${WIN_USER:-$USER}/Downloads/Quick Share/$GEMMA_MODEL" \
        "/media/windows/Users/${WIN_USER:-$USER}/Downloads/Quick Share/$GEMMA_MODEL" \
        "${SUDO_USER:+/home/$SUDO_USER/Downloads/$GEMMA_MODEL}" \
        "$HOME/Downloads/$GEMMA_MODEL" \
        "/root/$GEMMA_MODEL"; do
        if [ -f "$candidate" ]; then
            echo "  -> Modèle local trouvé : $candidate"
            cp "$candidate" "$MODELS_DIR/$GEMMA_MODEL"
            break
        fi
    done
fi

if [ -f "$GEMMA_MODEL" ]; then
    echo "  -> Modèle Gemma 4 E2B prêt : $MODELS_DIR/$GEMMA_MODEL"
else
    echo "  [INFO] Veuillez copier '$GEMMA_MODEL' dans $MODELS_DIR/"
fi

echo "[4/4] Création du lanceur rapide 'ask-ai'..."
cat << 'EOF' > "$BIN_DIR/ask-ai"
#!/bin/sh
# Lanceur interactif Gastinel AI avec offload Vulkan et threads Core Ultra 5
MODEL="${1:-/opt/gastinel-ai/models/Gemma-4-E2B-Uncensored-HauhauCS-Aggressive-Q2_K_P.gguf}"
shift 2>/dev/null || true

# -ngl 99 : Tout décharger dans la VRAM partagée LPDDR5x via Vulkan
# -t 6    : Utiliser les 2 P-Cores + 4 E-Cores pour éviter la saturation thermique
exec llama-cli \
    -m "$MODEL" \
    -ngl 99 \
    -t 6 \
    -c 4096 \
    --color \
    -p "Tu es l'assistant local Gastinel propulsé par Gemma 4. Tu réponds de manière concise, intelligente et directe en français." \
    "$@"
EOF

chmod +x "$BIN_DIR/ask-ai"

# Création du service démon arrière-plan pour le lanceur Spotlight
cat << 'EOF' > /etc/init.d/gastinel-ai-daemon
#!/sbin/openrc-run
description="Gastinel Local AI 2B Background Daemon (Port 8080)"

depend() {
    after gastinel-zram
}

start() {
    ebegin "Démarrage du démon IA 2B local (Vulkan / Gemma 4 E2B)"
    start-stop-daemon --start --background \
        --make-pidfile --pidfile /run/gastinel-ai.pid \
        --exec /usr/local/bin/llama-server -- \
        -m /opt/gastinel-ai/models/Gemma-4-E2B-Uncensored-HauhauCS-Aggressive-Q2_K_P.gguf \
        -ngl 99 -t 6 -c 4096 --port 8080 --host 127.0.0.1
    eend $?
}

stop() {
    ebegin "Arrêt du démon IA"
    start-stop-daemon --stop --pidfile /run/gastinel-ai.pid
    eend $?
}
EOF

chmod +x /etc/init.d/gastinel-ai-daemon

echo "============================================================"
echo "   INSTALLATION IA TERMINÉE AVEC SUCCÈS !                   "
echo "   Commande CLI directe : ask-ai                            "
echo "   Démon local Web/API  : rc-service gastinel-ai-daemon start"
echo "============================================================"
