#!/usr/bin/env bash
set -e

removed=0

if [ -f /usr/local/bin/fowl ]; then
    sudo rm /usr/local/bin/fowl
    echo "Removed /usr/local/bin/fowl"
    removed=1
fi

if [ -f "$HOME/.local/bin/fowl" ]; then
    rm "$HOME/.local/bin/fowl"
    echo "Removed $HOME/.local/bin/fowl"
    removed=1
fi

if [ $removed -eq 0 ]; then
    echo "fowl not found in /usr/local/bin or ~/.local/bin"
fi
