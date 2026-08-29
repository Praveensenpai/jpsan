#!/usr/bin/env bash
set -euo pipefail

REPO="Praveensenpai/jpsan"
BINARY="jpsan"
INSTALL_DIR="${HOME}/.local/bin"

# Ensure ~/.local/bin or ~/.cargo/bin is used
if [[ -d "${HOME}/.cargo/bin" ]] && [[ ":$PATH:" == *":${HOME}/.cargo/bin:"* ]]; then
    INSTALL_DIR="${HOME}/.cargo/bin"
fi
mkdir -p "${INSTALL_DIR}"

echo -e "\033[1;35m⛩️  Installing 浄化 (jpsan) — Anime Immersion Sanitizer...\033[0m"

# 1. If running inside local repository clone with Cargo.toml, build from source
if [[ -f "Cargo.toml" ]] && command -v cargo >/dev/null 2>&1; then
    echo -e "\033[1;34m• Building from local source...\033[0m"
    cargo build --release
    install -m 755 target/release/${BINARY} "${INSTALL_DIR}/${BINARY}"
    echo -e "\033[1;32m✔ Successfully installed jpsan to ${INSTALL_DIR}/${BINARY}\033[0m"
    echo -e "Run \033[1;36mjpsan --help\033[0m to get started! 🌸"
    exit 0
fi

# 2. Otherwise, download prebuilt binary from GitHub Releases
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [[ "${OS}" != "linux" ]]; then
    echo -e "\033[1;31m✖ Error: Prebuilt binaries are currently only provided for Linux (x86_64).\033[0m"
    exit 1
fi

if [[ "${ARCH}" != "x86_64" ]]; then
    echo -e "\033[1;31m✖ Error: Unsupported architecture ${ARCH}. Please build with cargo.\033[0m"
    exit 1
fi

ASSET="jpsan-linux-x86_64.tar.gz"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

echo -e "\033[1;34m• Downloading latest prebuilt release for ${OS}-${ARCH}...\033[0m"

# If GitHub CLI is available and authenticated (handles private repos seamlessly)
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    gh release download --repo "${REPO}" --pattern "${ASSET}" --dir "${TMP_DIR}"
else
    # Public release fallback via curl
    LATEST_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
    curl -fsSL "${LATEST_URL}" -o "${TMP_DIR}/${ASSET}"
fi

tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}"
install -m 755 "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"

echo -e "\033[1;32m✔ Successfully installed ${BINARY} to ${INSTALL_DIR}/${BINARY}\033[0m"
echo -e "Run \033[1;36mjpsan --help\033[0m to get started! 🌸"
