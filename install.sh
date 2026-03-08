#!/usr/bin/env bash
set -e

# Ensure rustup/cargo is available
if ! command -v cargo &>/dev/null; then
    echo "cargo not found — installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    source "$HOME/.cargo/env"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building fowl (this may take a minute on first run while aria2c is downloaded)..."
cargo build --release

BIN="$SCRIPT_DIR/target/release/fowl"

# Try system-wide install, fall back to user local
if sudo cp "$BIN" /usr/local/bin/fowl 2>/dev/null; then
    echo "Installed to /usr/local/bin/fowl"
else
    LOCAL_BIN="$HOME/.local/bin"
    mkdir -p "$LOCAL_BIN"
    cp "$BIN" "$LOCAL_BIN/fowl"
    echo "Installed to $LOCAL_BIN/fowl"
    # Hint if not in PATH
    if ! echo "$PATH" | tr ':' '\n' | grep -qx "$LOCAL_BIN"; then
        echo ""
        echo "Note: $LOCAL_BIN is not in your PATH."
        echo "Add this to your shell profile:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
fi

echo ""
echo "Run: fowl <url>"
