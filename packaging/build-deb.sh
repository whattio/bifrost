#!/usr/bin/env bash
# =============================================================================
# BIFROST — Native Debian / Ubuntu .deb Package Builder
# Produces: dist/bifrost-ghost-browser_<version>_<arch>.deb
# Installable via: sudo apt install ./dist/bifrost-ghost-browser_<version>_<arch>.deb
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

PKG_NAME="bifrost-ghost-browser"
VERSION="$(awk -F'"' '/^version =/{print $2; exit}' Cargo.toml)"
ARCH="$(dpkg --print-architecture)"
STAGE_DIR="$REPO_ROOT/dist/pkg_${PKG_NAME}_${VERSION}_${ARCH}"
OUT_DEB="$REPO_ROOT/dist/${PKG_NAME}_${VERSION}_${ARCH}.deb"

if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

echo "[1/3] Building release binary for ${ARCH}..."
cargo build --release

echo "[2/3] Staging Debian package tree (${PKG_NAME} v${VERSION} ${ARCH})..."
rm -rf "$STAGE_DIR"
mkdir -p \
  "$STAGE_DIR/DEBIAN" \
  "$STAGE_DIR/usr/bin" \
  "$STAGE_DIR/usr/share/applications" \
  "$STAGE_DIR/usr/share/icons/hicolor/256x256/apps" \
  "$STAGE_DIR/usr/share/icons/hicolor/512x512/apps" \
  "$STAGE_DIR/usr/share/pixmaps" \
  "$STAGE_DIR/usr/share/doc/${PKG_NAME}"

install -m 0755 "$REPO_ROOT/target/release/bifrost-ghost-browser" "$STAGE_DIR/usr/bin/bifrost-ghost-browser"
install -m 0644 "$REPO_ROOT/assets/bifrost-icon-256.png" "$STAGE_DIR/usr/share/icons/hicolor/256x256/apps/bifrost-ghost-browser.png"
install -m 0644 "$REPO_ROOT/assets/bifrost-icon-512.png" "$STAGE_DIR/usr/share/icons/hicolor/512x512/apps/bifrost-ghost-browser.png"
install -m 0644 "$REPO_ROOT/assets/bifrost-icon-256.png" "$STAGE_DIR/usr/share/pixmaps/bifrost-ghost-browser.png"
install -m 0644 "$REPO_ROOT/LICENSE" "$REPO_ROOT/NOTICE" "$REPO_ROOT/TRADEMARKS.md" "$REPO_ROOT/README.md" "$STAGE_DIR/usr/share/doc/${PKG_NAME}/"

cat <<'EOF' > "$STAGE_DIR/usr/share/applications/bifrost-ghost-browser.desktop"
[Desktop Entry]
Type=Application
Name=BIFROST
GenericName=The Reticulum Ghost Browser
Comment=Zero-IP Reticulum Ghost Browser (bifrost:// & ironpass://) — 0 Open Ports
Exec=/usr/bin/bifrost-ghost-browser --rns-listen 127.0.0.1:14290 %u
Icon=/usr/share/icons/hicolor/256x256/apps/bifrost-ghost-browser.png
Terminal=false
Categories=Network;WebBrowser;Security;
MimeType=x-scheme-handler/bifrost;x-scheme-handler/ironpass;x-scheme-handler/citadel;
StartupWMClass=bifrost-ghost-browser
StartupNotify=true
EOF

INSTALLED_KB="$(du -sk "$STAGE_DIR/usr" | awk '{print $1}')"

cat <<EOF > "$STAGE_DIR/DEBIAN/control"
Package: ${PKG_NAME}
Version: ${VERSION}
Section: net
Priority: optional
Architecture: ${ARCH}
Installed-Size: ${INSTALLED_KB}
Depends: libgtk-3-0t64 | libgtk-3-0, libwebkit2gtk-4.1-0, libc6
Maintainer: whatt.io <engineering@whatt.io>
Homepage: https://github.com/whattio/bifrost
Description: BIFROST — The Reticulum Ghost Browser (Zero-IP Sovereign Client)
 Native Zero-IP Reticulum browser (bifrost:// & ironpass://) built in
 clean-room Rust with 0 local HTTP ports, hard-blocked Web2 egress,
 RAM-only incognito sessions, and local Ed25519 + GS1 EPCIS 2.0
 IronPass Ledger verification.
EOF

cat <<'EOF' > "$STAGE_DIR/DEBIAN/postinst"
#!/bin/sh
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t /usr/share/icons/hicolor >/dev/null 2>&1 || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database /usr/share/applications >/dev/null 2>&1 || true
fi
exit 0
EOF
chmod 0755 "$STAGE_DIR/DEBIAN/postinst"

echo "[3/3] Packing ${OUT_DEB}..."
dpkg-deb --build --root-owner-group "$STAGE_DIR" "$OUT_DEB"
rm -rf "$STAGE_DIR"

echo "✅ Built: ${OUT_DEB}"
ls -lh "$OUT_DEB"
