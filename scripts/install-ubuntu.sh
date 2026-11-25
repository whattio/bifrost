#!/usr/bin/env bash
# =============================================================================
# BIFROST — The Reticulum Ghost Browser
# Ubuntu / Debian One-Command Installer & Uninstaller
#
# Installs:
#   - /usr/local/bin/bifrost-ghost-browser
#   - /usr/share/icons/hicolor/{256x256,512x512}/apps/bifrost-ghost-browser.png
#   - /usr/share/applications/bifrost-ghost-browser.desktop
#   - Registers bifrost://, ironpass://, and citadel:// URI handlers (xdg-mime)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DST="/usr/local/bin/bifrost-ghost-browser"
DESKTOP_DST="/usr/share/applications/bifrost-ghost-browser.desktop"
ICON_256="/usr/share/icons/hicolor/256x256/apps/bifrost-ghost-browser.png"
ICON_512="/usr/share/icons/hicolor/512x512/apps/bifrost-ghost-browser.png"

if [[ "${1:-}" == "--uninstall" ]]; then
  echo "[*] Uninstalling BIFROST — The Reticulum Ghost Browser..."
  sudo rm -f "$BIN_DST" "$DESKTOP_DST" "$ICON_256" "$ICON_512" /usr/share/pixmaps/bifrost-ghost-browser.png
  sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
  sudo update-desktop-database /usr/share/applications 2>/dev/null || true
  echo "[+] Uninstalled cleanly."
  exit 0
fi

echo "=================================================================="
echo "  ᛉ BIFROST — The Reticulum Ghost Browser (Ubuntu/Debian Installer)"
echo "=================================================================="

echo "[1/5] Installing required GTK3 & WebKitGTK 4.1 system packages..."
sudo apt-get update -qq
sudo apt-get install -y \
  build-essential pkg-config curl git \
  libgtk-3-dev libwebkit2gtk-4.1-dev \
  desktop-file-utils xdg-utils

if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "[2/5] Installing Rust toolchain via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
else
  echo "[2/5] Rust toolchain detected: $(rustc --version)"
fi

echo "[3/5] Building BIFROST release binary (clean-room Rust)..."
cd "$REPO_ROOT"
cargo build --release

echo "[4/5] Installing binary, desktop icons, and GNOME/freedesktop launcher..."
sudo install -m 0755 "$REPO_ROOT/target/release/bifrost-ghost-browser" "$BIN_DST"
sudo mkdir -p /usr/share/icons/hicolor/256x256/apps /usr/share/icons/hicolor/512x512/apps /usr/share/pixmaps
sudo install -m 0644 "$REPO_ROOT/assets/bifrost-icon-256.png" "$ICON_256"
sudo install -m 0644 "$REPO_ROOT/assets/bifrost-icon-512.png" "$ICON_512"
sudo install -m 0644 "$REPO_ROOT/assets/bifrost-icon-256.png" /usr/share/pixmaps/bifrost-ghost-browser.png

cat <<EOF | sudo tee "$DESKTOP_DST" >/dev/null
[Desktop Entry]
Type=Application
Name=BIFROST
GenericName=The Reticulum Ghost Browser
Comment=Zero-IP Reticulum Ghost Browser (bifrost:// & ironpass://) — 0 Open Ports
Exec=$BIN_DST --rns-listen 127.0.0.1:14290 %u
Icon=$ICON_256
Terminal=false
Categories=Network;WebBrowser;Security;
MimeType=x-scheme-handler/bifrost;x-scheme-handler/ironpass;x-scheme-handler/citadel;
StartupWMClass=bifrost-ghost-browser
StartupNotify=true
EOF

echo "[5/5] Updating icon cache & registering bifrost:// and ironpass:// URI handlers..."
sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
sudo update-desktop-database /usr/share/applications 2>/dev/null || true
xdg-mime default bifrost-ghost-browser.desktop \
  x-scheme-handler/bifrost x-scheme-handler/ironpass x-scheme-handler/citadel 2>/dev/null || true

echo ""
echo "✅ BIFROST installed!"
echo "   Launch from your Ubuntu App Grid (search 'BIFROST') or run:"
echo "   bifrost-ghost-browser --rns-listen 127.0.0.1:14290"
echo ""
echo "   Quick 60-second local demo:"
echo "   1) python3 $REPO_ROOT/examples/demo-vault/server.py"
echo "   2) citadel-vault-daemon --upstream http://127.0.0.1:8080 --rns-peer 127.0.0.1:14290"
echo "   3) bifrost-ghost-browser"
