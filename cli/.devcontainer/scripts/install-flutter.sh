#!/bin/bash
set -e

FLUTTER_DIR="/opt/flutter"
FLUTTER_BRANCH="${FLUTTER_BRANCH:-stable}"

echo "Clonando Flutter ${FLUTTER_BRANCH}..."
git clone https://github.com/flutter/flutter.git -b "${FLUTTER_BRANCH}" --depth 1 "$FLUTTER_DIR"

export PATH="$PATH:${FLUTTER_DIR}/bin"

echo "Ejecutando flutter doctor..."
flutter doctor

echo "Aceptando licencias Android..."
flutter doctor --android-licenses || true

echo "Flutter instalado en ${FLUTTER_DIR}"
