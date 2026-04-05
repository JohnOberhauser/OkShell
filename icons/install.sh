#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

rsync -a --delete "$SCRIPT_DIR/OkMaterial" "$HOME/.local/share/icons/"
rsync -a --delete "$SCRIPT_DIR/OkPhosphor" "$HOME/.local/share/icons/"