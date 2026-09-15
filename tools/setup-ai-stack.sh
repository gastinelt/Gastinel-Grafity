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

echo "[3/4] Téléchargement des modèles IA 2B ultra-rapides recommandés..."
cd "$MODELS_DIR"

# 1. Qwen 2.5 1.5B Instruct Q8_0 (~1.8 Go VRAM, français fluide, code, 70+ tok/s)
if [ ! -f "qwen2.5-1.5b-instruct-q8_0.gguf" ]; then
    echo "  -> Téléchargement de Qwen 2.5 1.5B Instruct (Q8_0)..."
    curl -L -C - -o qwen2.5-1.5b-instruct-q8_0.gguf \
        "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q8_0.gguf"
fi

# 2. DeepSeek R1 Distill Qwen 1.5B Q5_K_M (~1.2 Go VRAM, raisonnement logique)
if [ ! -f "DeepSeek-R1-Distill-Qwen-1.5B-Q5_K_M.gguf" ]; then
    echo "  -> Téléchargement de DeepSeek-R1-Distill 1.5B (Q5_K_M)..."
    curl -L -C - -o DeepSeek-R1-Distill-Qwen-1.5B-Q5_K_M.gguf \
        "https://huggingface.co/unsloth/DeepSeek-R1-Distill-Qwen-1.5B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-1.5B-Q5_K_M.gguf"
fi

echo "[4/4] Création du lanceur rapide 'ask-ai'..."
cat << 'EOF' > "$BIN_DIR/ask-ai"
#!/bin/sh
# Lanceur interactif Gastinel AI avec offload Vulkan et threads Core Ultra 5
MODEL="${1:-/opt/gastinel-ai/models/qwen2.5-1.5b-instruct-q8_0.gguf}"
shift 2>/dev/null || true

# -ngl 99 : Tout décharger dans la VRAM partagée LPDDR5x via Vulkan
# -t 6    : Utiliser les 2 P-Cores + 4 E-Cores pour éviter la saturation thermique
exec llama-cli \
    -m "$MODEL" \
    -ngl 99 \
    -t 6 \
    -c 4096 \
    --color \
    -p "Tu es l'assistant local Gastinel. Tu réponds de manière concise, intelligente et directe en français." \
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
    ebegin "Démarrage du démon IA 2B local (Vulkan / Qwen 2.5)"
    start-stop-daemon --start --background \
        --make-pidfile --pidfile /run/gastinel-ai.pid \
        --exec /usr/local/bin/llama-server -- \
        -m /opt/gastinel-ai/models/qwen2.5-1.5b-instruct-q8_0.gguf \
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
