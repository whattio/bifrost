#!/usr/bin/env python3
"""
BIFROST // Self-Hosted Zero-Port Ghost Demo Vault (Reference Example)
Runs strictly on 127.0.0.1:8080 (0 external ports). Expose ONLY over encrypted Reticulum:
  citadel-vault-daemon --identity-name my-ghost-vault --upstream http://127.0.0.1:8080 --rns-peer 127.0.0.1:14290

Endpoints:
  GET  /                     -> Sample Sovereign Zero-IP Ghost Webpage (bifrost://<hash>/)
  GET  /dpp/demo-01          -> Ed25519-signed EU ESPR DPP + hash-chained GS1 EPCIS 2.0 IronPass Ledger
  GET  /dpp/demo-01/ledger   -> Read the hash-chained EPCIS 2.0 ledger
  POST /dpp/demo-01/ledger   -> Append & Ed25519-sign a new GS1 EPCIS 2.0 event
"""
import hashlib
import json
import os
import subprocess
import time
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HOST = os.environ.get("BIFROST_DEMO_BIND", "127.0.0.1")
PORT = int(os.environ.get("BIFROST_DEMO_PORT", "8080"))
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
KEY_PATH = os.path.join(BASE_DIR, "demo_ed25519.pem")
LEDGER_DIR = os.path.join(BASE_DIR, ".ledger")
_PUB_HEX = None
_TAP_CTR = 1


def ensure_key():
    global _PUB_HEX
    if _PUB_HEX:
        return _PUB_HEX
    if not os.path.isfile(KEY_PATH):
        subprocess.run(["openssl", "genpkey", "-algorithm", "ED25519", "-out", KEY_PATH], check=True)
        os.chmod(KEY_PATH, 0o600)
    der = subprocess.check_output(["openssl", "pkey", "-in", KEY_PATH, "-pubout", "-outform", "DER"])
    _PUB_HEX = der[-32:].hex()
    return _PUB_HEX


def sign_hex(msg: str):
    pub = ensure_key()
    t_in = f"/tmp/bf_in_{os.getpid()}.bin"
    t_sig = f"/tmp/bf_sig_{os.getpid()}.bin"
    with open(t_in, "wb") as f:
        f.write(msg.encode("ascii"))
    subprocess.run(["openssl", "pkeyutl", "-sign", "-inkey", KEY_PATH, "-rawin", "-in", t_in, "-out", t_sig], check=True)
    with open(t_sig, "rb") as f:
        sig = f.read().hex()
    for p in (t_in, t_sig):
        try:
            os.remove(p)
        except OSError:
            pass
    return pub, sig


def build_event(seq: int, prev_hash: str, payload: dict) -> dict:
    core = {
        "seq": int(seq),
        "type": str(payload.get("type") or "ObjectEvent"),
        "eventTime": str(payload.get("eventTime") or time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())),
        "action": str(payload.get("action") or "OBSERVE").upper(),
        "bizStep": str(payload.get("bizStep") or "inspecting").lower(),
        "disposition": str(payload.get("disposition") or "active").lower(),
        "readPoint": str(payload.get("readPoint") or "urn:epc:id:sgln:0000000.00001.lab"),
        "actor": str(payload.get("actor") or "Authorized Bifrost Operator"),
        "note": str(payload.get("note") or ""),
        "prev_hash": prev_hash,
    }
    canon = json.dumps(core, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    ev_hash = hashlib.sha256(canon.encode("utf-8")).hexdigest()
    pub, sig = sign_hex(ev_hash)
    ev = dict(core)
    ev["event_hash"] = ev_hash
    ev["ed25519_pubkey_hex"] = pub
    ev["ed25519_signature_hex"] = sig
    return ev


def load_ledger(dpp_id: str):
    os.makedirs(LEDGER_DIR, exist_ok=True)
    path = os.path.join(LEDGER_DIR, f"{dpp_id}.jsonl")
    if not os.path.isfile(path):
        seeds = [
            {"type": "ObjectEvent", "action": "ADD", "bizStep": "commissioning", "disposition": "active",
             "readPoint": "urn:epc:id:sgln:0000000.00001.forge", "actor": "Local Forge Vault",
             "note": "Asset commissioned and bound to local Ed25519 signing key."},
            {"type": "ObjectEvent", "action": "OBSERVE", "bizStep": "inspecting", "disposition": "conformant",
             "readPoint": "urn:epc:id:sgln:0000000.00002.qa", "actor": "QA Verifier",
             "note": "EU ESPR circularity metrics verified with zero cloud exposure."},
        ]
        prev = "0" * 64
        out = []
        with open(path, "w", encoding="utf-8") as f:
            for i, s in enumerate(seeds, 1):
                ev = build_event(i, prev, s)
                prev = ev["event_hash"]
                out.append(ev)
                f.write(json.dumps(ev) + "\n")
        return out
    with open(path, "r", encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


SAMPLE_DPP = {
    "id": "demo-01",
    "product_number": "DEMO-ESPR-001",
    "name": "Sovereign Industrial Battery Pack (Demo)",
    "description": "Self-hosted sample EU ESPR Digital Product Passport served strictly over encrypted Reticulum with 0 open HTTP ports.",
    "material_description": "Recycled aluminum housing (72%), modular LFP cells, zero conflict cobalt.",
    "assembly_desc": "Assembled and cryptographically sealed inside a 0-port Linux vault.",
    "spare_parts_description": "Field-replaceable BMS board and cell modules with 10-year spares commitment.",
    "recycle_description": "Disassemble via 6 Torx fasteners; 94% material recovery rate.",
    "template_fields": [
        {"name": "EU ESPR Sustainability", "fields": [
            {"id": "espr_recycled_materials", "name": "Recycled Content", "value": "72", "unit": "%"},
            {"id": "espr_durability", "name": "Service Lifetime", "value": "12", "unit": "yrs"},
            {"id": "espr_mtbf", "name": "MTBF Reliability", "value": "50000", "unit": "h"},
            {"id": "espr_carbon", "name": "Carbon Footprint", "value": "18.4", "unit": "kg CO2e"},
        ]}
    ],
}

INDEX_HTML = """<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Zero-Port Ghost Demo Site</title>
<style>body{background:#020503;color:#f2faf5;font-family:system-ui,sans-serif;padding:42px;max-width:780px;margin:0 auto;line-height:1.6}
code{background:#08160d;padding:2px 6px;border-radius:6px;color:#bef264}
a{color:#38bdf8;font-weight:700}</style></head>
<body>
<h1>ᛉ Hello from a 0-Port Reticulum Ghost Server</h1>
<p>This webpage is bound strictly to <code>127.0.0.1:8080</code> and reached exclusively inside <strong>BIFROST — The Reticulum Ghost Browser</strong> over an encrypted X25519 + Ed25519 Reticulum link — zero DNS, zero public IP, zero open inbound ports.</p>
<p><strong>Next:</strong> Open the signed EU ESPR DPP + hash-chained GS1 EPCIS 2.0 Ledger on this exact node using <code>ironpass://&lt;your-hash&gt;/dpp/demo-01</code>.</p>
</body></html>"""


class Handler(BaseHTTPRequestHandler):
    def send_json(self, code, obj):
        body = json.dumps(obj, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        global _TAP_CTR
        p = urllib.parse.urlsplit(self.path).path
        if p in ("/", "/index.html"):
            b = INDEX_HTML.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(b)))
            self.end_headers()
            self.wfile.write(b)
            return
        if p == "/api/status":
            self.send_json(200, {"ok": True, "mode": "bifrost-demo-vault", "open_ports": 0})
            return
        if p == "/dpp/demo-01/ledger":
            evs = load_ledger("demo-01")
            self.send_json(200, {"ok": True, "dpp_id": "demo-01", "count": len(evs), "events": evs})
            return
        if p == "/dpp/demo-01":
            _TAP_CTR += 1
            canon = json.dumps(SAMPLE_DPP, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
            sha = hashlib.sha256(canon.encode("utf-8")).hexdigest()
            pub, sig = sign_hex(sha)
            self.send_json(200, {
                "protocol": "IronPass-Ghost-DPP/1.0",
                "dpp_id": "demo-01",
                "data": SAMPLE_DPP,
                "ironpass_ledger": load_ledger("demo-01"),
                "ironpass_proof": {
                    "vault_host": "local-demo-vault",
                    "vault_open_ports": 0,
                    "canonical_sha256": sha,
                    "ed25519_pubkey_hex": pub,
                    "ed25519_signature_hex": sig,
                    "ntag424_dna": {
                        "chip": "Simulated NXP NTAG 424 DNA (Demo)",
                        "uid": "04:DE:M0:00:00:00:80",
                        "read_counter": _TAP_CTR,
                        "cmac_status": "VERIFIED_VAULT_PROOF",
                        "tamper_loop": "INTACT",
                    },
                    "signed_at": int(time.time()),
                },
            })
            return
        self.send_json(404, {"ok": False, "error": f"Not found: {p}"})

    def do_POST(self):
        p = urllib.parse.urlsplit(self.path).path
        n = int(self.headers.get("Content-Length", "0") or "0")
        raw = self.rfile.read(n) if n > 0 else b"{}"
        if p == "/dpp/demo-01/ledger":
            body = json.loads(raw.decode("utf-8") or "{}")
            existing = load_ledger("demo-01")
            ev = build_event(len(existing) + 1, existing[-1]["event_hash"] if existing else "0" * 64, body)
            with open(os.path.join(LEDGER_DIR, "demo-01.jsonl"), "a", encoding="utf-8") as f:
                f.write(json.dumps(ev) + "\n")
            self.send_json(200, {"ok": True, "dpp_id": "demo-01", "event": ev})
            return
        self.send_json(404, {"ok": False})


if __name__ == "__main__":
    print(f"[*] BIFROST Demo Vault listening strictly on http://{HOST}:{PORT} (0 external ports)")
    ThreadingHTTPServer((HOST, PORT), Handler).serve_forever()
