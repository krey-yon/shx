#!/usr/bin/env bash
#
# Install shx into /usr/local/bin.
#
# Run this from inside the extracted shx-linux folder that came out of a release
# tarball:
#
#     tar -xzf shx-<target>.tar.gz
#     cd shx-linux
#     ./install.sh
#
# To install straight from source instead, run `cargo install --path .` yourself;
# this script is for the prebuilt binary.

set -euo pipefail

APP_NAME="shx"
BIN_DIR="${SHX_BIN_DIR:-/usr/local/bin}"
BIN_TARGET="${BIN_DIR}/${APP_NAME}"

if [[ "${EUID}" -eq 0 ]]; then
    echo "Do not run this script as root; it will call sudo when it needs it." >&2
    exit 1
fi

if ! command -v "${APP_NAME}" >/dev/null 2>&1; then
    if [[ ! -f "./${APP_NAME}" ]]; then
        echo "No '${APP_NAME}' binary in $(pwd)." >&2
        echo "Run this script from inside the extracted '${APP_NAME}-linux' folder," >&2
        echo "or install from source with: cargo install --path ." >&2
        exit 1
    fi
    # Prefer an installer-relative binary over one already on PATH, otherwise
    # this script would happily install the previously installed version over
    # itself.
    SOURCE_BINARY="./${APP_NAME}"
else
    SOURCE_BINARY="$(command -v "${APP_NAME}")"
fi

sudo mkdir -p "${BIN_DIR}"
sudo install -m 755 "${SOURCE_BINARY}" "${BIN_TARGET}"

echo "Installed ${APP_NAME} to ${BIN_TARGET}"
echo "Start it with: ${APP_NAME}"
echo
echo "On first run shx will ask for a model provider API key and store it in"
echo "~/.shx/config.json with owner-only permissions."
