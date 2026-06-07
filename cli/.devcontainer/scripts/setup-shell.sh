#!/bin/bash
set -e
REMOTE_USER="${REMOTE_USER:-developer}"
USER_HOME="/home/${REMOTE_USER}"

echo 'export PATH=$PATH:$HOME/.local/bin' >> "${USER_HOME}/.bashrc"
echo 'export JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64' >> "${USER_HOME}/.bashrc"
echo 'export ANDROID_SDK_ROOT=/usr/local/android-sdk' >> "${USER_HOME}/.bashrc"
echo 'export PATH=$PATH:$ANDROID_SDK_ROOT/cmdline-tools/bin:$ANDROID_SDK_ROOT/platform-tools' >> "${USER_HOME}/.bashrc"
echo 'export PATH=$PATH:/opt/flutter/bin' >> "${USER_HOME}/.bashrc"
