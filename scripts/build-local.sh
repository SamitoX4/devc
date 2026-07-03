#!/bin/bash
# build-local.sh — Compila devc e instala el binario + TUS templates LOCALES,
# sin bajar nada de GitHub. Pensado para desarrollo local.
#
# Diferencia con build-release.sh: aquel descarga las templates de GitHub master;
# este usa tu carpeta local  ../devcontainers/templates.
#
# Uso:
#   bash devc/scripts/build-local.sh                  # compila + instala binario + templates
#   bash devc/scripts/build-local.sh --templates-only # solo re-sincroniza templates (sin compilar)

set -e
export PATH="$HOME/.cargo/bin:$PATH"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"           # .../devc/scripts
CLI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)/cli"           # .../devc/cli
PROJECTS_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"       # .../AI Projects
TEMPLATES_SRC="$PROJECTS_DIR/devcontainers/templates" # tus templates locales

BIN_DEST="$HOME/.local/bin/devc"
CACHE_DEST="$HOME/.devc/cache/templates"

TEMPLATES_ONLY=false
[ "$1" = "--templates-only" ] && TEMPLATES_ONLY=true

# --- Validaciones ---
if [ ! -d "$TEMPLATES_SRC/nodejs" ]; then
    echo "✗ No encuentro templates locales en: $TEMPLATES_SRC"
    exit 1
fi

# --- Build (salvo --templates-only) ---
if [ "$TEMPLATES_ONLY" = false ]; then
    if ! command -v cargo >/dev/null 2>&1; then
        echo "✗ Falta cargo (Rust). Instala: https://rustup.rs"
        exit 1
    fi
    echo "==> Compilando devc (cargo build --release)... (puede tardar la 1ª vez)"
    ( cd "$CLI_DIR" && cargo build --release )
    BUILT_BIN="$CLI_DIR/target/release/devc"
    if [ ! -x "$BUILT_BIN" ]; then
        echo "✗ Build falló: no existe $BUILT_BIN"
        exit 1
    fi

    echo "==> Instalando binario en $BIN_DEST..."
    mkdir -p "$(dirname "$BIN_DEST")"
    if [ -f "$BIN_DEST" ]; then
        cp "$BIN_DEST" "$BIN_DEST.bak"
        echo "    backup del binario anterior -> $BIN_DEST.bak"
    fi
    install -m 0755 "$BUILT_BIN" "$BIN_DEST"
fi

# --- Instalar templates locales en la cache de devc ---
echo "==> Instalando templates locales en $CACHE_DEST..."
if [ -d "$CACHE_DEST" ]; then
    rm -rf "$CACHE_DEST.bak"
    mv "$CACHE_DEST" "$CACHE_DEST.bak"
    echo "    backup de la cache anterior -> $CACHE_DEST.bak"
fi
mkdir -p "$CACHE_DEST"
cp -r "$TEMPLATES_SRC/"* "$CACHE_DEST/"

# --- Verificación ---
echo
echo "==> Verificación:"
echo -n "    "; "$BIN_DEST" --version 2>/dev/null || echo "(no se pudo ejecutar el binario)"
echo "    templates disponibles:"
"$BIN_DEST" list 2>/dev/null | sed 's/^/      /' || ls -1 "$CACHE_DEST" | sed 's/^/      /'

echo
echo "✓ Listo: tu 'devc' usa el binario recién compilado + tus templates locales."
echo "  OJO: 'devc update' re-descarga templates de GitHub y sobrescribe la cache."
echo "       Para volver a tus locales tras un update:"
echo "         bash $SCRIPT_DIR/build-local.sh --templates-only"
