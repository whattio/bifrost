#!/usr/bin/env bash
# =============================================================================
# BIFROST — Zero-IP Ghost Browser · Powered by Reticulum
# Native macOS One-Command Installer & App Packager (Apple Silicon & Intel)
#
# Installs:
#   - /Applications/BIFROST.app (native WKWebView bundle + Retina .icns icon)
#   - Registers bifrost://, ironpass://, citadel://, and bifrost-card:// URIs
#   - Optional: saves outbound Reticulum relay to ~/.config/bifrost/peers.json
#   - Prints this Mac's persistent 32-hex Ed25519 Client ID (Lock 2)
#
# Usage:
#   ./scripts/install-macos.sh [--relay HOST:PORT] [--uninstall]
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_DST="/Applications/BIFROST.app"
RELAY_ADDR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --uninstall)
      echo "[*] Uninstalling BIFROST from /Applications/BIFROST.app..."
      rm -rf "$APP_DST"
      echo "[+] Uninstalled. (Your ~/.config/bifrost/ identity & contacts were preserved.)"
      exit 0
      ;;
    --relay)
      RELAY_ADDR="${2:-}"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "[!] This installer is for macOS (Darwin). On Linux, use ./scripts/install-ubuntu.sh or ./scripts/install-debian-pi.sh."
  exit 1
fi

echo "====================================================================="
echo "  BIFROST — Zero-IP Ghost Browser · Powered by Reticulum (macOS)"
echo "====================================================================="

if ! xcode-select -p >/dev/null 2>&1; then
  echo "[!] Xcode Command Line Tools not found. Triggering install..."
  xcode-select --install || true
  echo "[!] Re-run this script after Command Line Tools finish installing."
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "[*] Installing Rust toolchain via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

echo "[*] Building native BIFROST binary (target-cpu=native)..."
cd "$REPO_ROOT"
RUSTFLAGS="-C target-cpu=native" cargo build --release

BIN_SRC="$REPO_ROOT/target/release/bifrost-ghost-browser"
if [[ ! -x "$BIN_SRC" ]]; then
  echo "[!] Expected binary not found at $BIN_SRC"
  exit 1
fi

echo "[*] Packaging $APP_DST..."
TMP_APP="$(mktemp -d)/BIFROST.app"
mkdir -p "$TMP_APP/Contents/MacOS" "$TMP_APP/Contents/Resources"

cp "$BIN_SRC" "$TMP_APP/Contents/MacOS/bifrost-ghost-browser"
chmod +x "$TMP_APP/Contents/MacOS/bifrost-ghost-browser"

# Generate native Retina .icns from 512px PNG using built-in sips + iconutil
ICON_SRC="$REPO_ROOT/assets/bifrost-icon-512.png"
if [[ -f "$ICON_SRC" ]] && command -v sips >/dev/null 2>&1 && command -v iconutil >/dev/null 2>&1; then
  ICONSET_DIR="$(mktemp -d)/BIFROST.iconset"
  mkdir -p "$ICONSET_DIR"
  sips -z 16 16     "$ICON_SRC" --out "$ICONSET_DIR/icon_16x16.png"      >/dev/null
  sips -z 32 32     "$ICON_SRC" --out "$ICONSET_DIR/icon_16x16@2x.png"   >/dev/null
  sips -z 32 32     "$ICON_SRC" --out "$ICONSET_DIR/icon_32x32.png"      >/dev/null
  sips -z 64 64     "$ICON_SRC" --out "$ICONSET_DIR/icon_32x32@2x.png"   >/dev/null
  sips -z 128 128   "$ICON_SRC" --out "$ICONSET_DIR/icon_128x128.png"    >/dev/null
  sips -z 256 256   "$ICON_SRC" --out "$ICONSET_DIR/icon_128x128@2x.png" >/dev/null
  sips -z 256 256   "$ICON_SRC" --out "$ICONSET_DIR/icon_256x256.png"    >/dev/null
  cp "$ICON_SRC"                      "$ICONSET_DIR/icon_256x256@2x.png"
  cp "$ICON_SRC"                      "$ICONSET_DIR/icon_512x512.png"
  iconutil -c icns "$ICONSET_DIR" -o "$TMP_APP/Contents/Resources/BIFROST.icns" || true
  rm -rf "$(dirname "$ICONSET_DIR")"
fi

VERSION="$(grep -m1 '^version' "$REPO_ROOT/Cargo.toml" | sed -E 's/.*"([^"]+)".*/\1/' || echo "0.3.5")"

cat > "$TMP_APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>BIFROST</string>
  <key>CFBundleDisplayName</key>
  <string>BIFROST</string>
  <key>CFBundleIdentifier</key>
  <string>io.whatt.bifrost</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleExecutable</key>
  <string>bifrost-ghost-browser</string>
  <key>CFBundleIconFile</key>
  <string>BIFROST</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSCameraUsageDescription</key>
  <string>BIFROST uses your camera strictly offline (0 cloud calls) to scan IronPass Hybrid #ip= QR codes.</string>
  <key>CFBundleURLTypes</key>
  <array>
    <dict>
      <key>CFBundleURLName</key>
      <string>BIFROST Zero-IP URI</string>
      <key>CFBundleURLSchemes</key>
      <array>
        <string>bifrost</string>
        <string>ironpass</string>
        <string>citadel</string>
        <string>bifrost-card</string>
      </array>
    </dict>
  </array>
</dict>
</plist>
EOF

# Ad-hoc sign so macOS Gatekeeper / Apple Silicon allows smooth local launch
codesign --force --deep --sign - "$TMP_APP" 2>/dev/null || true

rm -rf "$APP_DST" 2>/dev/null || sudo rm -rf "$APP_DST"
mv "$TMP_APP" "$APP_DST" 2>/dev/null || sudo mv "$TMP_APP" "$APP_DST"
rm -rf "$(dirname "$TMP_APP")"

# Optionally pre-configure an outbound Reticulum relay
if [[ -n "$RELAY_ADDR" ]]; then
  mkdir -p "$HOME/.config/bifrost"
  if [[ ! -f "$HOME/.config/bifrost/peers.json" ]]; then
    printf '["%s"]\n' "$RELAY_ADDR" > "$HOME/.config/bifrost/peers.json"
    echo "[+] Saved outbound Reticulum relay $RELAY_ADDR to ~/.config/bifrost/peers.json"
  fi
fi

# Register URI schemes with LaunchServices
LSREG="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
if [[ -x "$LSREG" ]]; then
  "$LSREG" -f "$APP_DST" || true
fi

echo ""
echo "====================================================================="
echo "  [+] Installed: $APP_DST"
echo "  [+] Your Lock 2 Ed25519 Device Client ID:"
"$APP_DST/Contents/MacOS/bifrost-ghost-browser" --print-client-id || true
echo "====================================================================="
echo "  Launch with: open /Applications/BIFROST.app"
