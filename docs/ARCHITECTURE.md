# BIFROST — Architecture & Zero-Port Threat Model

## 1. The Last-Meter Leak Problem
Modern Digital Product Passport initiatives (EU ESPR, CIRPASS-2, W3C Verifiable Credentials) sign data payloads, yet still deliver them through conventional HTTPS web browsers (Chrome, Safari, Edge). This leaves three structural leaks:

1. **Metadata & Pattern-of-Life Exposure:** DNS lookups, source/destination IP headers, TLS SNI hostnames, and CDN logs expose who scanned which asset serial number, where, and when — even when JSON payloads are encrypted.
2. **Hostile Client Footprint:** Mainstream browsers ship tens of millions of lines of ad-tech/extension runtime, write history/cache/cookies to SSD, and phone home telemetry to third-party cloud platforms.
3. **Forced Server Attack Surface:** Standard browsers only speak `https://`, forcing asset vaults to expose public IPs, DNS records, and port `:443` (or terminate TLS on a third-party cloud CDN).

## 2. How BIFROST Closes the Last Meter

| Security Layer | Mechanism |
| :--- | :--- |
| **In-Process Reticulum Transport** | `citadel-rns-rs` runs directly inside the `bifrost-ghost-browser` Rust process via custom URI protocol handlers (`bifrost://`, `ironpass://`, `citadel://`). **0 localhost HTTP sockets** are opened. |
| **End-to-End Cryptography** | 512-bit Curve25519 identity keysets (`X25519` ephemeral ECDH + `Ed25519` signatures) and 128-bit cryptographic destination hashes over `FastTcp` (`TCP_NODELAY` + $O(1)$ HDLC). |
| **Hard-Blocked Web2 Egress** | `with_navigation_handler` drops every `http://` and `https://` request by default (`[BIFROST-DROP]`). |
| **RAM-Only Incognito WebView** | `WebViewBuilder::with_incognito(true)` paired with an in-memory `localStorage`/`sessionStorage` shim ensures zero browser state persists to disk. |
| **Local Rust Verification** | `ironpass://` pulls strictly signed JSON (zero remote HTML/JS) and verifies canonical SHA-256 + RFC 8032 Ed25519 signatures and hash-chained GS1 EPCIS 2.0 ledgers locally via `ed25519-dalek` + `sha2`. |
| **Infrastructure Neutrality** | Runs over any untrusted transport pipe — Fiber, Starlink, 5G, Wi-Fi, or out-of-band Sub-GHz LoRa (~1.7 kbps @ SF9 for ~1.7 KB DPPs & emergency C2). |

## 3. Wire Flow (CHT2 over Reticulum)
1. A 0-port Linux Ghost Vault (`nftables` `policy drop` inbound) dials outbound via `citadel-vault-daemon --rns-peer <BIFROST_IP>:14290`.
2. BIFROST receives the Reticulum announce (optionally restricted by `--allow-hash <128-bit-hash>`) and establishes an ephemeral X25519 + Ed25519 link.
3. Custom scheme requests (`bifrost://` or `ironpass://`) are chunked into 400-byte `CHT2` frames with synchronized `WINDOW_SIZE = 10` flow control and fast gzip compression (decompressed in RAM inside Rust off the broadcast loop).
4. Responses render directly into WebKitGTK / WKWebView / WebView2 with zero TCP HTTP proxy hops.
