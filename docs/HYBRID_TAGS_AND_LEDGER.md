# Hybrid `#ip=` Tags & IronPass Ledger (GS1 EPCIS 2.0) Specification

## 1. Hybrid Public + Classified Tag URI (`#ip=`)
One physical QR code or NFC tag (NTAG 215 / NTAG 424 DNA) serves two completely isolated worlds:

```text
https://example.org/dpp/DEMO-ESPR-001#ip=<128-bit-rns-hash>/dpp/demo-01
|<------ Public Consumer Tier ------>||<---- Classified Ghost Tier ---->|
```

- **Consumer Tap (Any Smartphone):** Per **RFC 3986**, URI `#fragment` identifiers are **never transmitted** to web servers by browsers. The consumer opens the public EU ESPR sustainability card with zero app installation, while the `#ip=` Reticulum destination hash never leaves the phone.
- **Authorized Operator Scan (BIFROST):** BIFROST detects `#ip=`, discards the `https://` prefix locally without issuing a DNS query or HTTP request, and opens `ironpass://<128-bit-rns-hash>/dpp/demo-01` directly over encrypted Reticulum.

## 2. Signed DPP Envelope (`GET /dpp/<id>`)
BIFROST expects a JSON document with:
- `data`: Standard EU ESPR / product passport JSON object.
- `ironpass_proof`:
  - `canonical_sha256`: Hex SHA-256 of `canonical_json(data)` (`sort_keys=True, separators=(",", ":")`).
  - `ed25519_pubkey_hex`: 32-byte raw Ed25519 public key (64 hex chars).
  - `ed25519_signature_hex`: 64-byte RFC 8032 Ed25519 signature over the ASCII hex `canonical_sha256` string.
  - `ntag424_dna`: Optional hardware silicon attestation metadata (`uid`, monotonic `read_counter`, `cmac_status`, `tamper_loop`).
- `ironpass_ledger`: Optional ordered array of hash-chained GS1 EPCIS 2.0 events.

## 3. IronPass Ledger — Hash-Chained GS1 EPCIS 2.0 Events
Each event in `ironpass_ledger` (and `GET/POST /dpp/<id>/ledger`) contains:

```json
{
  "seq": 1,
  "type": "ObjectEvent",
  "eventTime": "2026-11-24T08:15:00Z",
  "action": "ADD",
  "bizStep": "commissioning",
  "disposition": "active",
  "readPoint": "urn:epc:id:sgln:0000000.00001.forge",
  "actor": "Local Forge Vault",
  "note": "Asset commissioned and bound to Ed25519 key.",
  "prev_hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "event_hash": "<sha256(canonical_json(core_10_fields))>",
  "ed25519_pubkey_hex": "<64-hex-pubkey>",
  "ed25519_signature_hex": "<128-hex-signature>"
}
```

### Independent Local Rust Verification (`src/dpp.rs`)
For every event `i` (`1..=N`), BIFROST independently verifies:
1. `seq == i` (strictly monotonic sequence).
2. `prev_hash == event[i-1].event_hash` (or 64 ASCII `'0'` characters for `seq == 1`).
3. `SHA-256(canonical_json(core_10_fields)) == event_hash`.
4. `Ed25519::verify_strict(pubkey, event_hash.as_bytes(), signature) == Ok(())`.

If any single byte of any historical event is altered, reordered, or deleted, BIFROST flags **`TAMPER DETECTED`** immediately.
