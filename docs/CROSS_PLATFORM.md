# Cross-Platform Build & Roadmap

Because **BIFROST** is built on Rust (`wry` + `tao` + `citadel-rns-rs` + `ed25519-dalek`), the core cryptographic transport, custom URI handlers, and local Ed25519 / EPCIS Ledger verifier are **100% portable** across desktop and mobile operating systems.

| Platform | WebView Engine | Status | Notes |
| :--- | :--- | :--- | :--- |
| **Ubuntu / Debian Linux (`x86_64` & `aarch64`)** | WebKitGTK 4.1 (`libwebkit2gtk-4.1`) | **Ready Today (`v0.1.0`)** | Native `.deb` package + `scripts/install-ubuntu.sh` + `xdg-mime` URI handlers |
| **macOS (Apple Silicon & Intel)** | Native Apple `WKWebView` | **Source-Ready (Testing)** | `src/main.rs` already gates `.build_gtk(...)` vs `.build(&window)` via `#[cfg(target_os = "linux")]`. Package via `cargo bundle` into `.app`/`.dmg` with `CFBundleURLTypes` for `bifrost://` & `ironpass://`. |
| **Windows 10 / 11** | Native Microsoft `WebView2` | **Source-Ready (Testing)** | Compiles directly with MSVC toolchain (`cargo build --release`) into a ~5 MB `.exe` + MSI/NSIS installer registering `bifrost://` & `ironpass://`. |
| **iOS & Android** | Tauri 2.0 Mobile (`wry` + Native NFC) | **Planned (`v0.2.0`)** | Embeds the exact same `citadel-rns-rs` + `dpp.rs` Rust core inside Tauri 2.0 Mobile with native CoreNFC / Android NFC NTAG 424 DNA tap & camera QR scanning directly into `ironpass://`. |

## Building on Linux
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev pkg-config build-essential
cargo build --release
```

## Building on macOS (Preview)
```bash
cargo build --release
./target/release/bifrost-ghost-browser --rns-listen 127.0.0.1:14290
```

## Building on Windows (Preview)
Ensure Microsoft Edge WebView2 Runtime is installed (included on Windows 11):
```powershell
cargo build --release
.\target\release\bifrost-ghost-browser.exe --rns-listen 127.0.0.1:14290
```
