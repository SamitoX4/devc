#!/bin/bash
set -e

ANDROID_SDK_ROOT=${ANDROID_SDK_ROOT:-/usr/local/android-sdk}
CMD_LINE_TOOLS_VERSION="${CMDLINE_TOOLS_VERSION:-7302050_latest}"
PLATFORMS_VERSION="android-${ANDROID_API_LEVEL:-34}"
BUILD_TOOLS_VERSION="${BUILD_TOOLS_VERSION:-34.0.0}"
NDK_VERSION="${NDK_VERSION:-26.1.10909125}"

TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo "Descargando Android command line tools..."
wget -q "https://dl.google.com/android/repository/commandlinetools-linux-${CMD_LINE_TOOLS_VERSION}.zip" -O cmdline-tools.zip

echo "Extrayendo en ${ANDROID_SDK_ROOT}..."
mkdir -p "${ANDROID_SDK_ROOT}"
unzip -q cmdline-tools.zip -d "${ANDROID_SDK_ROOT}"
rm -f cmdline-tools.zip

export PATH="$PATH:${ANDROID_SDK_ROOT}/cmdline-tools/bin"

echo "Aceptando licencias y actualizando SDK..."
yes | sdkmanager --sdk_root="${ANDROID_SDK_ROOT}" --update

echo "Instalando platform-tools, NDK, platforms y build-tools..."
yes | sdkmanager --sdk_root="${ANDROID_SDK_ROOT}" --install "platform-tools"
yes | sdkmanager --sdk_root="${ANDROID_SDK_ROOT}" --install "ndk;${NDK_VERSION}"
yes | sdkmanager --sdk_root="${ANDROID_SDK_ROOT}" --install "platforms;${PLATFORMS_VERSION}"
yes | sdkmanager --sdk_root="${ANDROID_SDK_ROOT}" --install "build-tools;${BUILD_TOOLS_VERSION}"

rm -rf "$TMP_DIR"

echo "Android SDK instalado correctamente en ${ANDROID_SDK_ROOT}"
