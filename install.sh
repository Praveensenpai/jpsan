#!/usr/bin/env bash
set -euo pipefail

echo -e "\033[1;35m⛩️  Installing 浄化 (jpsan) — Anime Immersion Sanitizer...\033[0m"

cargo build --release

INSTALL_DIR="${HOME}/.cargo/bin"
mkdir -p "${INSTALL_DIR}"
install -m 755 target/release/jpsan "${INSTALL_DIR}/jpsan"

echo -e "\033[1;32m✔ Successfully installed jpsan to ${INSTALL_DIR}/jpsan\033[0m"
echo -e "Run \033[1;36mjpsan --help\033[0m to get started!"
