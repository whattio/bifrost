<p align="center">
  <img src="assets/bifrost-icon-256.png" width="128" height="128" alt="BIFROST — The Reticulum Ghost Browser" />
</p>

<h1 align="center">BIFROST</h1>
<p align="center"><strong>The Reticulum Ghost Browser</strong></p>
<p align="center">
  <em>“Verifiable Credentials protect the passport. Chrome leaks the mission.”</em>
</p>
<p align="center">
  <code>bifrost://</code> &nbsp;·&nbsp; <code>ironpass://</code> &nbsp;·&nbsp; <strong>0 Local HTTP Ports</strong> &nbsp;·&nbsp; <strong>0 DNS</strong> &nbsp;·&nbsp; <strong>0 Cookies</strong> &nbsp;·&nbsp; <strong>Clean-Room Rust</strong>
</p>

---

## What Is BIFROST?

**BIFROST** (*The Reticulum Ghost Browser*) is **not** a replacement for Chrome. It is a **5 MB Sovereign Zero-IP Client** built in clean-room Rust (`wry` + `tao` + [`citadel-rns-rs`](https://github.com/whattio/citadel-rns-rs)) that closes **“The Last-Meter Leak”** in **CIRPASS-2**, **EU ESPR Digital Product Passports**, and sovereign zero-IP infrastructure:

- **Zero Local HTTP Ports:** Embeds `citadel-rns-rs` directly in-process — zero localhost web proxy sockets (`ss -tlnp` shows no listening HTTP servers).
- **Zero Web2 / DNS Egress:** Hard-blocks `http://` and `https://` navigation by default. All traffic rides over encrypted **512-bit Curve25519 (X25519 ECDH + Ed25519)** Reticulum links across any untrusted physical pipe (Fiber, Starlink, 5G, Wi-Fi, or Sub-GHz LoRa).
- **RAM-Only Ephemeral Sessions:** Runs in incognito mode with an in-memory `localStorage`/`sessionStorage` shim — zero cookies, zero history, and zero cache written to SSD.
- **Local Rust Ed25519 + GS1 EPCIS 2.0 Verification:** Opens pure `~1.7 KB` signed JSON passports over `ironpass://`, verifies both the canonical **SHA-256 + RFC 8032 Ed25519** signature and the entire hash-chained **IronPass Ledger** locally via `ed25519-dalek` in **~30 ms** — never trusting remote JavaScript.
- **Hybrid Tag (`#ip=`) Dual-Tier Support:** Consumers tap an NFC/QR tag with any normal phone for the public EU ESPR passport (zero app needed); authorized operators scan the exact same tag in **BIFROST** to strip Web2 locally and unlock the classified vault & EPCIS ledger over Reticulum.
- **Optional Local Sovereign AI Omnibox:** Connect your own local OpenAI-compatible NPU / Ollama / `llama.cpp` server (e.g. Qualcomm Ventuno Q 40 TOPS NPU) over Reticulum for zero-cloud AI queries directly from `bifrost://home/`.

---

## Executive Contrast: CIRPASS-2 in Chrome vs. BIFROST

| Layer | Standard Web2 DPP (CIRPASS-2 in Chrome) | BIFROST (`bifrost://` & `ironpass://`) |
| :--- | :--- | :--- |
| **Compliance / Schema** | EU ESPR, W3C VC, GS1 EPCIS 2.0 | **100% compatible** (identical EU ESPR & GS1 EPCIS 2.0 JSON) |
| **Network Exposure** | Public IP + DNS + open `:443` + Cloud CDN | **0 open server ports · 0 local HTTP sockets · 0 DNS · 0 cloud** |
| **Browser Footprint** | Cookies, SSD history/cache, extensions, telemetry | **RAM-only ephemeral session · Web2 (`http`/`https`) hard-blocked** |
| **Verification** | JavaScript downloaded from the web server | **Local Clean-Room Rust (`ed25519-dalek` + `sha2` + NTAG 424 DNA)** |
| **Payload & Resilience** | Heavy web app — fails when internet/cloud is down | **Pure ~1.7 KB signed JSON** — Fiber, Starlink, 5G & 1.7 kbps Sub-GHz LoRa |
| **Consumer UX** | Regular phone tap | **Hybrid Tag (`#ip=`)**: zero-app consumer tap + classified Ghost operator tier |

---

## Architecture Overview

```mermaid
flowchart LR
    Tag["Hybrid QR / NFC Tag\nhttps://example.org/dpp/01#ip=<hash>/dpp/demo-01"]
    Phone["Consumer Smartphone\n(Ignores #ip= locally)"]
    Web["Public EU ESPR Page\n(Public Transparency Only)"]
    Bifrost["BIFROST Ghost Browser\n(0 HTTP Ports · RAM-Only · Web2 Blocked)"]
    RNS["Encrypted Reticulum Link\n(X25519 + Ed25519 · Fiber / Starlink / 5G / LoRa)"]
    Vault["0-Port Linux Ghost Vault\n(nftables DROP · 127.0.0.1 Upstream)"]

    Tag -->|Regular Tap| Phone --> Web
    Tag -->|Authorized Scan| Bifrost -->|Strips https:// locally| RNS --> Vault
```

---

## Quickstart (Ubuntu / Debian Linux)

### Option 1 — Install via `.deb` Package (Recommended, 2 seconds)
Download the latest `.deb` from **Releases** (or build it yourself with `./packaging/build-deb.sh`):
```bash
sudo apt install ./bifrost-ghost-browser_0.1.0_amd64.deb
```

### Option 2 — One-Command Build & Install Script
Installs GTK/WebKit2GTK dev headers, compiles **BIFROST**, installs the binary + desktop icon, and registers `bifrost://` and `ironpass://` URI handlers in Ubuntu:
```bash
git clone https://github.com/whattio/bifrost.git
cd bifrost
./scripts/install-ubuntu.sh
```

### Option 3 — 60-Second Self-Hosted Local Demo (Zero External Servers!)
BIFROST ships with **zero central servers and zero hardcoded remote hosts**. Test a complete 0-port Ghost Website + Ed25519 DPP + hash-chained **IronPass Ledger** strictly on your own machine:

1. **Terminal 1 — Start the localhost-only demo vault (`127.0.0.1:8080`, 0 external ports):**
   ```bash
   python3 examples/demo-vault/server.py
   ```
2. **Terminal 2 — Expose it strictly over encrypted Reticulum (`citadel-rns-rs`):**
   ```bash
   cargo install --git https://github.com/whattio/citadel-rns-rs.git --bin citadel-vault-daemon
   citadel-vault-daemon --identity-name my-demo-vault --upstream http://127.0.0.1:8080 --rns-peer 127.0.0.1:14290
   ```
3. **Terminal 3 — Launch BIFROST:**
   ```bash
   bifrost-ghost-browser --rns-listen 127.0.0.1:14290
   ```
   Open **📡 NODES** inside BIFROST — click **OPEN GHOST SITE** (`bifrost://<hash>/`) or **OPEN DEMO DPP + LEDGER** (`ironpass://<hash>/dpp/demo-01`), then seal a new Ed25519-signed GS1 EPCIS 2.0 event live!

---

## Operator Bookmarks & Local Sovereign AI (Optional)
- **Private Bookmarks (`~/.config/bifrost/bookmarks.json`):** Add your own node shortcuts locally without ever committing hashes to source control:
  ```json
  [
    { "label": "My Ghost Vault", "icon": "🔒", "uri": "bifrost://<32-hex-hash>/", "tone": "holo" },
    { "label": "Battery DPP", "icon": "📜", "uri": "ironpass://<32-hex-hash>/dpp/demo-01", "tone": "amber" }
  ]
  ```
- **Local AI Omnibox (Ollama / `llama.cpp` / Ventuno Q):** Run `citadel-vault-daemon --upstream http://127.0.0.1:11434 --rns-peer <BIFROST_IP>:14290` (or port `8090` for `llama-server`), place any optional Bearer token in `~/.config/bifrost/llm_api_key` (`chmod 600`), and launch with `--llm-hash <hash> --llm-model <model>`.

---

## Documentation
- [**Architecture & Zero-Port Threat Model**](docs/ARCHITECTURE.md)
- [**Hybrid `#ip=` Tags & IronPass Ledger (GS1 EPCIS 2.0) Specification**](docs/HYBRID_TAGS_AND_LEDGER.md)
- [**Hosting a 0-Port Ghost Web & DPP Server**](docs/HOSTING_A_GHOST_SERVER.md)
- [**Cross-Platform Roadmap (Linux, macOS, Windows & Mobile)**](docs/CROSS_PLATFORM.md)
- [**Security Policy**](SECURITY.md)

---

## Licensing, Attribution & Trademarks
- **Source Code:** Dual-licensed under [**GNU AGPLv3 (`AGPL-3.0-or-later`)**](LICENSE) or a [**Commercial License**](COMMERCIAL-LICENSE.md) for OEM, white-label, or closed-source redistribution.
- **Wire Protocol Attribution:** Built on the **Public Domain Reticulum Wire Protocol** invented by **Mark Qvist** (<https://reticulum.network>), using clean-room MIT Rust crates (`citadel-rns-rs`) — see [**NOTICE**](NOTICE).
- **Trademarks & Icons:** `BIFROST`, `CITADEL`, `IRONPASS`, `ASGAARD`, and all icons in `assets/` are protected — see [**TRADEMARKS.md**](TRADEMARKS.md).
