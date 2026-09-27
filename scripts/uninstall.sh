#!/usr/bin/env bash
#
# Remove shx. Optionally also removes the config file, which contains your API
# key, so that is an explicit opt-in rather than a default.

set -euo pipefail

APP_NAME="shx"
BIN_DIR="${SHX_BIN_DIR:-/usr/local/bin}"
BIN_TARGET="${BIN_DIR}/${APP_NAME}"
CONFIG_FILE="${HOME}/.shx/config.json"
HISTORY_FILE="${HOME}/.shx_history"

if [[ "${EUID}" -eq 0 ]]; then
    echo "Do not run this script as root; it will call sudo when it needs it." >&2
    exit 1
fi

echo "Uninstalling ${APP_NAME}..."
sudo rm -f "${BIN_TARGET}"

read -r -p "Remove ${CONFIG_FILE} (contains your API key)? [y/N]: " reply
if [[ "${reply}" =~ ^[Yy]$ ]]; then
    rm -f "${CONFIG_FILE}"
    echo "Config removed."
fi

read -r -p "Remove ${HISTORY_FILE}? [y/N]: " reply
if [[ "${reply}" =~ ^[Yy]$ ]]; then
    rm -f "${HISTORY_FILE}"
    echo "History removed."
fi

echo "${APP_NAME} uninstalled."
