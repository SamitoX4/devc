FROM ubuntu:24.04
ARG REMOTE_USER=root
ARG REMOTE_PASSWORD=S?I.c_5QPFMk
ARG CONTAINER_PASSWORD=S?I.c_5QPFMk
ARG SUDO_MODE=none
ENV REMOTE_USER=${REMOTE_USER}
ENV REMOTE_PASSWORD=${REMOTE_PASSWORD}
ENV CONTAINER_PASSWORD=${CONTAINER_PASSWORD}
ENV SUDO_MODE=${SUDO_MODE}

# Build arguments for version customization
ARG ANDROID_API_LEVEL=34
ARG BUILD_TOOLS_VERSION=34.0.0
ARG NDK_VERSION=26.1.10909125
ARG CMDLINE_TOOLS_VERSION=7302050_latest
ARG FLUTTER_BRANCH=stable

# Expose as environment variables so scripts can read them
ENV ANDROID_API_LEVEL=${ANDROID_API_LEVEL}
ENV BUILD_TOOLS_VERSION=${BUILD_TOOLS_VERSION}
ENV NDK_VERSION=${NDK_VERSION}
ENV CMDLINE_TOOLS_VERSION=${CMDLINE_TOOLS_VERSION}
ENV FLUTTER_BRANCH=${FLUTTER_BRANCH}
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y passwd sudo

RUN echo "root:${CONTAINER_PASSWORD}" | chpasswd

RUN if [ "${REMOTE_USER}" != "root" ] && ! id "${REMOTE_USER}" &>/dev/null; then \
        useradd -m -s /bin/bash "${REMOTE_USER}"; \
    fi && \
    if [ "${REMOTE_USER}" != "root" ]; then \
        echo "${REMOTE_USER}:${REMOTE_PASSWORD}" | chpasswd; \
    fi

RUN if [ "${SUDO_MODE}" != "none" ] && [ "${REMOTE_USER}" != "root" ]; then \
        if [ "${SUDO_MODE}" = "nopasswd" ]; then \
            echo "${REMOTE_USER} ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/${REMOTE_USER}; \
        else \
            echo "${REMOTE_USER} ALL=(ALL) ALL" > /etc/sudoers.d/${REMOTE_USER}; \
        fi && \
        chmod 0440 /etc/sudoers.d/${REMOTE_USER}; \
    fi

COPY --chown=root:root ./scripts/install-system-packages.sh /tmp/install-system-packages.sh
RUN chmod +x /tmp/install-system-packages.sh && /tmp/install-system-packages.sh && rm /tmp/install-system-packages.sh

COPY --chown=root:root ./scripts/install-android.sh /tmp/install-android.sh
RUN chmod +x /tmp/install-android.sh && /tmp/install-android.sh && rm /tmp/install-android.sh

COPY --chown=root:root ./scripts/install-flutter.sh /tmp/install-flutter.sh
RUN chmod +x /tmp/install-flutter.sh && /tmp/install-flutter.sh && rm /tmp/install-flutter.sh

COPY --chown=root:root ./scripts/install-shared.sh /tmp/install-shared.sh
RUN chmod +x /tmp/install-shared.sh && /tmp/install-shared.sh && rm /tmp/install-shared.sh

COPY --chown=root:root ./scripts/setup-shell.sh /root/setup-shell.sh
RUN chmod +x /root/setup-shell.sh && /root/setup-shell.sh

WORKDIR /workspace
