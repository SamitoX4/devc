#!/bin/bash
set -e

apt-get update && apt-get install -y --no-install-recommends \
    openjdk-17-jdk \
    wget \
    unzip \
    git \
    adb \
    curl \
    lsb-release \
    gnupg \
    software-properties-common \
    zsh \
    zsh-common \
    fonts-firacode \
    fonts-powerline \
    python3 \
    python3-pip \
    ca-certificates \
    libc6 \
    libstdc++6 \
    xz-utils \
    zip \
    libglu1-mesa \
    chromium-browser

rm -rf /var/lib/apt/lists/*
