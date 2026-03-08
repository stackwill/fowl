#!/usr/bin/env bash
set -e

REPO="stackwill/fowl"
BIN_NAME="fowl"

# Detect install destination
if [ -w /usr/local/bin ]; then
    INSTALL_DIR="/usr/local/bin"
elif sudo -n true 2>/dev/null; then
    INSTALL_DIR="/usr/local/bin"
    USE_SUDO=1
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$BIN_NAME"

echo "Downloading fowl..."
if command -v curl &>/dev/null; then
    curl -fsSL "$DOWNLOAD_URL" -o "/tmp/$BIN_NAME"
elif command -v wget &>/dev/null; then
    wget -qO "/tmp/$BIN_NAME" "$DOWNLOAD_URL"
else
    echo "Error: curl or wget required" >&2
    exit 1
fi

chmod +x "/tmp/$BIN_NAME"

if [ "${USE_SUDO:-0}" = "1" ]; then
    sudo mv "/tmp/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
    mv "/tmp/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
fi

echo "Installed to $INSTALL_DIR/$BIN_NAME"

# Hint if install dir is not in PATH
if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    echo ""
    echo "Note: $INSTALL_DIR is not in your PATH."
    echo "Add this to your shell profile:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
fi

echo ""
echo "Run: fowl <url>"
