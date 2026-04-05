cargo install --path okshell
cargo install --path okshellctl
cargo install --path okshellshare

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

rsync -a --delete "$SCRIPT_DIR/icons/OkMaterial" "$HOME/.local/share/icons/"
rsync -a --delete "$SCRIPT_DIR/icons/OkPhosphor" "$HOME/.local/share/icons/"