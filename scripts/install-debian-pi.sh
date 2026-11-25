#!/usr/bin/env bash
# =============================================================================
# BIFROST — The Reticulum Ghost Browser
# Debian 12/13 & Raspberry Pi OS (64-bit arm64 / amd64) Installer
#
# Modes:
#   ./scripts/install-debian-pi.sh                      # Build & install from source
#   ./scripts/install-debian-pi.sh --deb <file.deb>     # Fast 2-second prebuilt .deb install (recommended for Pi!)
#   ./scripts/install-debian-pi.sh --uninstall          # Clean removal
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DST="/usr/local/bin/bifrost-ghost-browser"
DESKTOP_DST="/usr/share/applications/bifrost-ghost-browser.desktop"
ICON_256="/usr/share/icons/hicolor/256x256/apps/bifrost-ghost-browser.png"
ICON_512="/usr/share/icons/hicolor/512x512/apps/bifrost-ghost-browser.png"

# Detect Raspberry Pi hardware (WebKitGTK DMA-BUF workaround for Pi 4/5 Wayland KMS)
IS_RPI=0
if [[ -r /proc/device-tree/model ]] && grep -qi "raspberry pi" /proc/device-tree/model 2>/dev/null; then
  IS_RPI=1
fi

if [[ "${1:-}" == "--uninstall" ]]; then
  echo "[*] Uninstalling BIFROST — The Reticulum Ghost Browser..."
  sudo apt-get remove -y bifrost-ghost-browser 2>/dev/null || true
  sudo rm -f "$BIN_DST" "$DESKTOP_DST" "$ICON_256" "$ICON_512" /usr/share/pixmaps/bifrost-ghost-browser.png
  sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
  sudo update-desktop-database /usr/share/applications 2>/dev/null || true
  echo "[+] Uninstalled cleanly."
  exit 0
fi

echo "=================================================================="
echo "  ᛉ BIFROST — Debian & Raspberry Pi OS Installer ($(dpkg --print-architecture))"
echo "=================================================================="

# Mode 1: Fast prebuilt .deb install (ideal for Raspberry Pi 4/5 — zero Rust compiling!)
if [[ "${1:-}" == "--deb" ]]; then
  DEB_FILE="${2:-}"
  if [[ -z "$DEB_FILE" || ! -f "$DEB_FILE" ]]; then
    echo "[!] Usage: $0 --deb /path/to/bifrost-ghost-browser_0.1.0_arm64.deb" >&2
    exit 1
  fi
  echo "[1/3] Installing prebuilt Debian package: $DEB_FILE ..."
  sudo apt-get update -qq
  sudo apt-get install -y "$DEB_FILE" xdg-utils desktop-file-utils
  if [[ "$IS_RPI" -eq 1 ]]; then
    echo "[2/3] Raspberry Pi detected — enabling WEBKIT_DISABLE_DMABUF_RENDERER=1 in launcher..."
    sudo sed -i 's|^Exec=/usr/bin/bifrost-ghost-browser|Exec=env WEBKIT_DISABLE_DMABUF_RENDERER=1 /usr/bin/bifrost-ghost-browser|' "$DESKTOP_DST"
  else
    echo "[2/3] Standard Debian/ARM64 GPU detected — using native WebKitGTK acceleration."
  fi
  echo "[3/3] Registering bifrost:// and ironpass:// URI handlers..."
  xdg-mime default bifrost-ghost-browser.desktop \
    x-scheme-handler/bifrost x-scheme-handler/ironpass x-scheme-handler/citadel 2>/dev/null || true
  echo "✅ BIFROST installed via .deb! Launch from your Applications menu or run: bifrost-ghost-browser"
  exit 0
fi

# Mode 2: Native build & install from source (Debian 12 Bookworm / 13 Trixie / Raspberry Pi OS 64-bit)
echo "[1/5] Installing Debian GTK3 & WebKitGTK 4.1 dependencies..."
sudo apt-get update -qq
sudo apt-get install -y \
  build-essential pkg-config curl git ca-certificates \
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

echo "[3/5] Building BIFROST release binary..."
cd "$REPO_ROOT"
# Limit parallel jobs on low-RAM boards (< 4 GB) to avoid OOM
MEM_KB="$(awk '/MemTotal:/ {print $2}' /proc/meminfo 2>/dev/null || echo 4194304)"
if [[ "$MEM_KB" -lt 4000000 ]]; then
  echo "    (Low-RAM board detected — building with -j 2 to prevent OOM)"
  cargo build --release -j 2
else
  cargo build --release
fi

echo "[4/5] Installing binary, desktop icons, and freedesktop launcher..."
sudo install -m 0755 "$REPO_ROOT/target/release/bifrost-ghost-browser" "$BIN_DST"
sudo mkdir -p /usr/share/icons/hicolor/256x256/apps /usr/share/icons/hicolor/512x512/apps /usr/share/pixmaps
sudo install -m 0644 "$REPO_ROOT/assets/bifrost-icon-256.png" "$ICON_256"
sudo install -m 0644 "$REPO_ROOT/assets/bifrost-icon-512.png" "$ICON_512"
sudo install -m 0644 "$REPO_ROOT/assets/bifrost-icon-256.png" /usr/share/pixmaps/bifrost-ghost-browser.png

EXEC_CMD="$BIN_DST --rns-listen 127.0.0.1:14290 %u"
if [[ "$IS_RPI" -eq 1 ]]; then
  echo "    (Raspberry Pi detected — enabling WEBKIT_DISABLE_DMABUF_RENDERER=1 for Wayland/KMS stability)"
  EXEC_CMD="env WEBKIT_DISABLE_DMABUF_RENDERER=1 $EXEC_CMD"
fi

cat <<EOF | sudo tee "$DESKTOP_DST" >/dev/null
[Desktop Entry]
Type=Application
Name=BIFROST
GenericName=The Reticulum Ghost Browser
Comment=Zero-IP Reticulum Ghost Browser (bifrost:// & ironpass://) — 0 Open Ports
Exec=$EXEC_CMD
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
echo "✅ BIFROST installed cleanly on Debian / Raspberry Pi!"
echo "   Launch from the Applications menu (search 'BIFROST') or run: bifrost-ghost-browser"
