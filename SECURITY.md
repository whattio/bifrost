# Security Policy — BIFROST

## Security Guarantees
BIFROST is engineered as a **Zero-IP Sovereign Client**:
- **0 local HTTP ports:** No local web proxy sockets are bound on the operator workstation.
- **Hard-blocked Web2 egress:** `http://` and `https://` navigation is blocked by default (`--allow-web2` is strictly opt-in and off by default).
- **RAM-only ephemeral sessions:** WebView runs in incognito mode with an in-memory storage shim; zero cookies, browsing history, or disk cache are written to SSD.
- **Local Rust cryptography:** Ed25519 DPP signatures and SHA-256 hash-chained GS1 EPCIS 2.0 ledgers are verified natively in Rust (`ed25519-dalek` + `sha2`) — never trusting remote JavaScript.
- **Zero-secret repository hygiene:** Never pass API keys on shared CLI command lines if other users can inspect `ps aux`; place local NPU keys in `~/.config/bifrost/llm_api_key` (`chmod 600`).

## Reporting a Vulnerability
Please **do not** open public GitHub issues for security vulnerabilities (especially WebView sandbox escapes, URI parser bypasses, or cryptographic verification flaws).

Report privately via:
- GitHub Security Advisories on <https://github.com/whattio/bifrost/security/advisories>
- Email: <mailto:security@whatt.io>

We aim to acknowledge reports within 48 hours and publish coordinated advisories once patched.
