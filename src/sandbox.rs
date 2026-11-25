//! Built-in Zero-Setup Local Showcase & Signed EU ESPR DPP + IronPass Ledger Sandbox
//!
//! Lets anyone who installs BIFROST immediately explore:
//!   1. `bifrost://home/showcase` — readable self-contained BIFROST / CITADEL / IRONPASS example microsite
//!   2. `ironpass://local-demo/dpp/{battery-01,drone-01,furniture-01}` — 3 realistic EU ESPR DPPs
//!      with real RFC 8032 Ed25519 signatures and SHA-256 hash-chained GS1 EPCIS 2.0 IronPass Ledgers
//!      (supporting live POST appends into the RAM-only sandbox ledger), verified by the exact same
//!      local Rust cryptographic verifier used for remote 0-port Reticulum vaults.

use ed25519_dalek::{Signer, SigningKey};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

struct SandboxState {
    signing_key: SigningKey,
    pubkey_hex: String,
    ledgers: Mutex<HashMap<String, Vec<Value>>>,
    tap_counters: Mutex<HashMap<String, u64>>,
}

static SANDBOX: OnceLock<SandboxState> = OnceLock::new();

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn canonical_json(val: &Value) -> String {
    match val {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut sorted = Map::new();
            for k in keys {
                let v = &map[k];
                let norm = match v {
                    Value::Object(_) | Value::Array(_) => {
                        serde_json::from_str(&canonical_json(v)).unwrap_or_else(|_| v.clone())
                    }
                    _ => v.clone(),
                };
                sorted.insert(k.clone(), norm);
            }
            serde_json::to_string(&Value::Object(sorted)).unwrap_or_default()
        }
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter().map(canonical_json).collect();
            format!("[{}]", parts.join(","))
        }
        _ => serde_json::to_string(val).unwrap_or_default(),
    }
}

fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex_encode(&h.finalize())
}

fn sign_event(
    sk: &SigningKey,
    pub_hex: &str,
    seq: u64,
    prev_hash: &str,
    ev_type: &str,
    event_time: &str,
    action: &str,
    biz_step: &str,
    disposition: &str,
    read_point: &str,
    actor: &str,
    note: &str,
) -> Value {
    let core = serde_json::json!({
        "seq": seq,
        "type": ev_type,
        "eventTime": event_time,
        "action": action,
        "bizStep": biz_step,
        "disposition": disposition,
        "readPoint": read_point,
        "actor": actor,
        "note": note,
        "prev_hash": prev_hash,
    });
    let ev_hash = sha256_hex(&canonical_json(&core));
    let sig = sk.sign(ev_hash.as_bytes());
    let sig_hex = hex_encode(&sig.to_bytes());
    serde_json::json!({
        "seq": seq,
        "type": ev_type,
        "eventTime": event_time,
        "action": action,
        "bizStep": biz_step,
        "disposition": disposition,
        "readPoint": read_point,
        "actor": actor,
        "note": note,
        "prev_hash": prev_hash,
        "event_hash": ev_hash,
        "ed25519_pubkey_hex": pub_hex,
        "ed25519_signature_hex": sig_hex,
    })
}

fn state() -> &'static SandboxState {
    SANDBOX.get_or_init(|| {
        let seed: [u8; 32] = Sha256::digest(b"bifrost-local-sandbox-ed25519-v1").into();
        let sk = SigningKey::from_bytes(&seed);
        let pub_hex = hex_encode(sk.verifying_key().as_bytes());
        let genesis = "0".repeat(64);

        let mut ledgers = HashMap::new();

        // 1. battery-01 (EU Battery Passport 2027)
        let b1 = sign_event(
            &sk, &pub_hex, 1, &genesis, "ObjectEvent", "2026-09-14T09:00:00Z", "ADD",
            "commissioning", "active", "urn:epc:id:sgln:7350000.00001.cell-line-04",
            "Nordic Gigafactory Forge", "LFP module serialized, NXP NTAG 424 DNA bound to Ed25519 root-of-trust.",
        );
        let b2 = sign_event(
            &sk, &pub_hex, 2, b1["event_hash"].as_str().unwrap(), "ObjectEvent", "2026-10-08T13:20:00Z", "OBSERVE",
            "inspecting", "conformant", "urn:epc:id:sgln:7350000.00002.espr-lab",
            "EU Battery Regulation Auditor", "State-of-health 100%, carbon footprint 58.2 kg CO2e/kWh verified.",
        );
        let b3 = sign_event(
            &sk, &pub_hex, 3, b2["event_hash"].as_str().unwrap(), "TransactionEvent", "2026-11-12T10:45:00Z", "OBSERVE",
            "receiving", "active", "urn:epc:id:sgln:7350000.00009.microgrid-02",
            "Grid Operator Custody Vault", "Installed in critical infrastructure microgrid; zero cloud telemetry.",
        );
        ledgers.insert("battery-01".to_string(), vec![b1, b2, b3]);

        // 2. drone-01 (Defense / Aerospace MRO UAV)
        let d1 = sign_event(
            &sk, &pub_hex, 1, &genesis, "ObjectEvent", "2026-10-01T08:15:00Z", "ADD",
            "commissioning", "active", "urn:epc:id:sgln:7350000.00010.airframe-01",
            "Airframe Assembly Vault", "Carbon composite airframe & flight controller hashed and sealed.",
        );
        let d2 = sign_event(
            &sk, &pub_hex, 2, d1["event_hash"].as_str().unwrap(), "ObjectEvent", "2026-11-05T16:30:00Z", "OBSERVE",
            "inspecting", "conformant", "urn:epc:id:sgln:7350000.00011.mro-hangar",
            "MRO Readiness Inspector", "Pre-deployment 50-hour rotor & battery inspection signed offline over Reticulum.",
        );
        ledgers.insert("drone-01".to_string(), vec![d1, d2]);

        // 3. furniture-01 (EU ESPR Industrial Furniture)
        let f1 = sign_event(
            &sk, &pub_hex, 1, &genesis, "ObjectEvent", "2026-08-22T11:00:00Z", "ADD",
            "commissioning", "active", "urn:epc:id:sgln:7350000.00020.mill-01",
            "Nordic Timber Workshop", "FSC-certified birch batch tagged with Hybrid #ip= consumer + Ghost QR.",
        );
        let f2 = sign_event(
            &sk, &pub_hex, 2, f1["event_hash"].as_str().unwrap(), "ObjectEvent", "2026-10-19T15:10:00Z", "OBSERVE",
            "inspecting", "conformant", "urn:epc:id:sgln:7350000.00021.espr-qa",
            "Circularity QA Verifier", "Modular repair parts & 92% recyclability verified under EU ESPR.",
        );
        ledgers.insert("furniture-01".to_string(), vec![f1, f2]);

        let mut counters = HashMap::new();
        counters.insert("battery-01".to_string(), 14);
        counters.insert("drone-01".to_string(), 41);
        counters.insert("furniture-01".to_string(), 9);

        SandboxState {
            signing_key: sk,
            pubkey_hex: pub_hex,
            ledgers: Mutex::new(ledgers),
            tap_counters: Mutex::new(counters),
        }
    })
}

fn sample_dpp_data(id: &str) -> Option<(Value, &'static str)> {
    match id {
        "battery-01" | "demo-01" => Some((
            serde_json::json!({
                "id": "battery-01",
                "product_number": "EU-BATT-LFP-48V-100",
                "name": "Nordic LFP Industrial Battery Pack (EU Battery Passport)",
                "description": "Local sandbox example of an EU 2027 Battery Passport served with 0 external servers. Demonstrates local Rust Ed25519 verification, Hybrid #ip= public/classified split, and a live hash-chained GS1 EPCIS 2.0 IronPass Ledger.",
                "material_description": "72% recycled aluminum enclosure, cobalt-free LiFePO4 cells, halogen-free BMS harness.",
                "assembly_desc": "Assembled in zero-emission Nordic facility; every cell lot cryptographically linked in IronPass Ledger.",
                "spare_parts_description": "Field-replaceable BMS controller, terminal busbars, and thermal sensors stocked for 12 years.",
                "recycle_description": "6 Torx fasteners for non-destructive disassembly; 94% hydrometallurgical recovery rate.",
                "template_fields": [
                    {
                        "name": "EU ESPR & Battery Regulation (2023/1542)",
                        "fields": [
                            { "id": "espr_recycled_materials", "name": "Recycled Content", "value": "72", "unit": "%" },
                            { "id": "espr_durability", "name": "Expected Service Life", "value": "12", "unit": "yrs" },
                            { "id": "espr_reliability", "name": "Cycle Life (80% SoH)", "value": "6000", "unit": "cycles" },
                            { "id": "espr_carbon", "name": "Lifecycle Carbon Footprint", "value": "58.2", "unit": "kg CO2e/kWh" }
                        ]
                    },
                    {
                        "name": "Classified Operator BOM (Ghost Tier Only)",
                        "fields": [
                            { "id": "bom_cells", "name": "Cell Supplier Lot", "value": "LOT-SE-2026-0914-A" },
                            { "id": "bom_bms_fw", "name": "Signed BMS Firmware SHA-256", "value": "9f82c4e1...7a10b3" },
                            { "id": "bom_custody", "name": "Current Authorized Owner", "value": "Sovereign Microgrid Ops" }
                        ]
                    }
                ]
            }),
            "04:BA:77:E9:12:48:80",
        )),
        "drone-01" => Some((
            serde_json::json!({
                "id": "drone-01",
                "product_number": "UAV-MRO-INSPECT-01",
                "name": "Autonomous Inspection UAV — MRO Grade (Sample)",
                "description": "Defense & critical-infrastructure MRO Digital Product Passport example. Demonstrates zero pattern-of-life metadata leakage (0 DNS, 0 IP, 0 cloud logs) when inspecting fleet hardware.",
                "material_description": "Recycled aerospace-grade carbon fiber frame (48%), magnesium gimbal chassis.",
                "assembly_desc": "Airframe, motors, and flight controller serialized and bound to NXP NTAG 424 DNA tamper silicon.",
                "spare_parts_description": "Quick-swap brushless arms, propellers, and modular battery bays rated for 15-minute field MRO.",
                "recycle_description": "Battery modules and PCB electronics separate cleanly for certified WEEE & aerospace recovery.",
                "template_fields": [
                    {
                        "name": "EU ESPR & Fleet MRO Metrics",
                        "fields": [
                            { "id": "espr_recycled_materials", "name": "Recycled Structural Content", "value": "48", "unit": "%" },
                            { "id": "espr_durability", "name": "Airframe Service Life", "value": "8", "unit": "yrs" },
                            { "id": "espr_reliability", "name": "MTBF Reliability", "value": "25000", "unit": "h" },
                            { "id": "espr_repair", "name": "Field Repairability Index", "value": "9.1", "unit": "/10" }
                        ]
                    }
                ]
            }),
            "04:E1:6A:92:4B:18:80",
        )),
        "furniture-01" => Some((
            serde_json::json!({
                "id": "furniture-01",
                "product_number": "ESPR-WOOD-DESK-2510",
                "name": "Nordic Birch Modular Desk (EU ESPR Sample)",
                "description": "European industrial OEM example showing how one Hybrid #ip= tag gives consumers a zero-app public EU ESPR passport while keeping supplier pricing, BOM, and ownership transfers private inside a 0-port vault.",
                "material_description": "100% FSC Nordic birch veneer, 65% recycled steel frame, zero-VOC bio-oil finish.",
                "assembly_desc": "Flat-pack precision joinery with zero permanent adhesives for lifetime refurbishment.",
                "spare_parts_description": "Every tabletop panel, bracket, and leg bolt is individually serialized for spare-part ordering.",
                "recycle_description": "Disassembles in under 4 minutes; 92% wood and steel recovery rate.",
                "template_fields": [
                    {
                        "name": "EU ESPR Circularity Metrics",
                        "fields": [
                            { "id": "espr_recycled_materials", "name": "Recycled Steel & Wood", "value": "65", "unit": "%" },
                            { "id": "espr_durability", "name": "Expected Service Life", "value": "20", "unit": "yrs" },
                            { "id": "espr_carbon", "name": "Carbon Footprint", "value": "24.5", "unit": "kg CO2e" }
                        ]
                    }
                ]
            }),
            "04:7C:3B:19:A2:51:80",
        )),
        _ => None,
    }
}

pub fn get_signed_sandbox_dpp_json(raw_id: &str) -> Option<Vec<u8>> {
    let key_id = if raw_id == "demo-01" { "battery-01" } else { raw_id };
    let (data_obj, uid) = sample_dpp_data(key_id)?;
    let st = state();

    let canonical = canonical_json(&data_obj);
    let sha = sha256_hex(&canonical);
    let sig = st.signing_key.sign(sha.as_bytes());
    let sig_hex = hex_encode(&sig.to_bytes());

    let ctr = {
        let mut map = st.tap_counters.lock().unwrap();
        let c = map.entry(key_id.to_string()).or_insert(1);
        *c += 1;
        *c
    };

    let ledger = st
        .ledgers
        .lock()
        .unwrap()
        .get(key_id)
        .cloned()
        .unwrap_or_default();

    let doc = serde_json::json!({
        "protocol": "IronPass-Ghost-DPP/1.0",
        "dpp_id": key_id,
        "data": data_obj,
        "ironpass_ledger": ledger,
        "ironpass_proof": {
            "vault_host": "local-sandbox (built-in RAM demo)",
            "vault_rns_hash": "local-demo",
            "vault_open_ports": 0,
            "canonical_sha256": sha,
            "ed25519_pubkey_hex": st.pubkey_hex,
            "ed25519_signature_hex": sig_hex,
            "ntag424_dna": {
                "chip": "Simulated NXP NTAG 424 DNA (Local Sandbox)",
                "uid": uid,
                "read_counter": ctr,
                "cmac_status": "VERIFIED_VAULT_PROOF",
                "tamper_loop": "INTACT"
            }
        }
    });

    serde_json::to_vec(&doc).ok()
}

pub fn append_sandbox_ledger_event(raw_id: &str, body: &[u8]) -> Result<String, String> {
    let key_id = if raw_id == "demo-01" { "battery-01" } else { raw_id };
    let req: Value = serde_json::from_slice(body).map_err(|e| format!("Invalid JSON: {e}"))?;
    let st = state();
    let mut map = st.ledgers.lock().unwrap();
    let list = map.entry(key_id.to_string()).or_default();
    let next_seq = (list.len() as u64) + 1;
    let prev_hash = list
        .last()
        .and_then(|v| v.get("event_hash"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "0".repeat(64));

    let ev = sign_event(
        &st.signing_key,
        &st.pubkey_hex,
        next_seq,
        &prev_hash,
        req.get("type").and_then(|v| v.as_str()).unwrap_or("ObjectEvent"),
        "2026-11-25T12:00:00Z",
        req.get("action").and_then(|v| v.as_str()).unwrap_or("OBSERVE"),
        req.get("bizStep").and_then(|v| v.as_str()).unwrap_or("inspecting"),
        req.get("disposition").and_then(|v| v.as_str()).unwrap_or("active"),
        req.get("readPoint").and_then(|v| v.as_str()).unwrap_or("urn:epc:id:sgln:7350000.00001.local"),
        req.get("actor").and_then(|v| v.as_str()).unwrap_or("Authorized Bifrost Operator"),
        req.get("note").and_then(|v| v.as_str()).unwrap_or("Sealed in local BIFROST sandbox"),
    );
    list.push(ev.clone());
    Ok(serde_json::json!({ "ok": true, "dpp_id": key_id, "event": ev }).to_string())
}

pub const SHOWCASE_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>BIFROST // LOCAL SOVEREIGN MICROSITE EXAMPLE</title>
<link rel="icon" type="image/png" href="bifrost://home/icons/citadel.png" />
<style>
  :root {
    --bg: #030705; --panel: rgba(8, 18, 12, 0.94); --panel2: rgba(12, 26, 18, 0.94);
    --lime: #84cc16; --emerald: #10b981; --holo: #38bdf8; --amber: #f59e0b; --red: #f87171;
    --text: #f2faf5; --muted: #9ab8a7; --border: rgba(16, 185, 129, 0.25);
  }
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    min-height: 100vh; color: var(--text);
    background: radial-gradient(ellipse 75% 45% at 50% 0%, rgba(132,204,22,0.12), transparent 60%), var(--bg);
    font-family: system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
    padding: 26px 28px 48px; line-height: 1.6;
  }
  .mono { font-family: ui-monospace, 'JetBrains Mono', SFMono-Regular, Menlo, monospace; }
  .wrap { max-width: 1180px; margin: 0 auto; display: flex; flex-direction: column; gap: 22px; }
  .topbar { display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px; border-bottom: 1px solid var(--border); padding-bottom: 14px; }
  .pill { display: inline-block; padding: 4px 10px; border-radius: 999px; font-size: 11px; font-weight: 800; border: 1px solid rgba(132,204,22,0.45); background: rgba(132,204,22,0.12); color: #d9f99d; }
  .pill.holo { border-color: rgba(56,189,248,0.45); background: rgba(56,189,248,0.12); color: #bae6fd; }
  .pill.amber { border-color: rgba(245,158,11,0.45); background: rgba(245,158,11,0.12); color: #fde68a; }
  .hero { background: var(--panel); border: 1px solid rgba(132,204,22,0.4); border-radius: 16px; padding: 28px 30px; display: flex; flex-direction: column; gap: 14px; }
  .hero h1 { font-size: clamp(26px, 3.4vw, 40px); font-weight: 800; line-height: 1.1; }
  .hero h1 span { color: var(--lime); }
  .quote { padding: 14px 18px; border-left: 4px solid var(--lime); background: rgba(132,204,22,0.10); border-radius: 8px; font-size: 20px; font-weight: 800; color: #f4fee8; }
  .quote em { font-style: normal; color: var(--red); }
  .grid3 { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; }
  @media (max-width: 860px) { .grid3 { grid-template-columns: 1fr; } }
  .card { background: var(--panel); border: 1px solid var(--border); border-radius: 14px; padding: 20px; display: flex; flex-direction: column; gap: 10px; }
  .card-head { display: flex; align-items: center; gap: 12px; }
  .card-head img { width: 52px; height: 52px; border-radius: 13px; border: 1px solid rgba(148,198,168,0.25); }
  .btn { display: inline-flex; align-items: center; gap: 6px; padding: 9px 14px; border-radius: 9px; font-weight: 800; font-size: 12px; text-decoration: none; cursor: pointer; background: rgba(132,204,22,0.16); color: #d9f99d; border: 1px solid rgba(132,204,22,0.5); }
  .btn:hover { background: rgba(132,204,22,0.28); color: #fff; }
  .btn.holo { background: rgba(56,189,248,0.14); color: #bae6fd; border-color: rgba(56,189,248,0.48); }
  .btn.amber { background: rgba(245,158,11,0.14); color: #fde68a; border-color: rgba(245,158,11,0.48); }
  table { width: 100%; border-collapse: collapse; font-size: 13.5px; }
  th, td { text-align: left; padding: 11px 13px; border-bottom: 1px solid rgba(16,185,129,0.15); }
  th { font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; background: rgba(2,8,5,0.9); }
  code { background: rgba(2,8,5,0.9); padding: 1px 6px; border-radius: 5px; color: #bae6fd; font-family: ui-monospace, monospace; font-size: 12px; }
</style>
</head>
<body>
<div class="wrap">
  <header class="topbar">
    <div style="display:flex; align-items:center; gap:12px;">
      <img src="bifrost://home/icons/bifrost.png" alt="BIFROST" style="width:42px;height:42px;border-radius:11px;" />
      <div>
        <div class="mono" style="font-size:15px; font-weight:800;">BIFROST // LOCAL OFFLINE MICROSITE EXAMPLE</div>
        <div class="mono" style="font-size:11px; color:var(--holo);">SERVED 100% LOCALLY FROM RUST RAM · 0 INTERNET · 0 OPEN PORTS · 0 COOKIES</div>
      </div>
    </div>
    <div style="display:flex; gap:8px; flex-wrap:wrap;" class="mono">
      <a class="btn" href="bifrost://home/">← BIFROST HOME</a>
      <a class="btn amber" href="ironpass://local-demo/dpp/battery-01">🔋 TRY BATTERY DPP + LEDGER →</a>
    </div>
  </header>

  <section class="hero">
    <div class="mono" style="display:flex; gap:8px; flex-wrap:wrap;">
      <span class="pill">BUILT-IN INTERACTIVE EXAMPLE</span>
      <span class="pill holo">CIRPASS-2 &amp; EU ESPR COMPATIBLE</span>
      <span class="pill amber">CLEAN-ROOM RUST · AGPL-3.0 / COMMERCIAL</span>
    </div>
    <h1>CLOSING THE LAST-METER LEAK.<br /><span>ZERO-IP WEBSITES &amp; IRONPASS DPPs.</span></h1>
    <div class="quote">“Verifiable Credentials protect the passport. <em>Chrome leaks the mission.</em>”</div>
    <p style="color:var(--muted); max-width:880px;">
      You are reading this page completely offline inside <strong>BIFROST — The Reticulum Ghost Browser</strong>.
      Zero DNS lookups were made, zero external servers were contacted, and zero cookies were written to disk.
      Use the live local examples below to test real Rust <strong>Ed25519 signature verification</strong>,
      <strong>Hybrid <code>#ip=</code> QR tags</strong>, and a hash-chained <strong>GS1 EPCIS 2.0 IronPass Ledger</strong> right now.
    </p>
    <div style="display:flex; gap:10px; flex-wrap:wrap;" class="mono">
      <a class="btn amber" href="ironpass://local-demo/dpp/battery-01">🔋 EU Battery Passport + Ledger →</a>
      <a class="btn holo" href="ironpass://local-demo/dpp/drone-01">🛸 Defense / MRO UAV DPP →</a>
      <a class="btn" href="ironpass://local-demo/dpp/furniture-01">🪑 EU ESPR Furniture DPP →</a>
      <a class="btn holo" href="bifrost://home/qr-sheet">🖨️ Printable Hybrid QR Sheet →</a>
    </div>
  </section>

  <section class="grid3">
    <article class="card">
      <div class="card-head">
        <img src="bifrost://home/icons/citadel.png" alt="Citadel" />
        <div>
          <span class="pill mono">0-PORT SERVER FABRIC</span>
          <h3 style="margin-top:4px;">CITADEL</h3>
        </div>
      </div>
      <p style="color:var(--muted); font-size:13.5px;">Turns any hardened Debian/Ubuntu Linux server into a 0-port Reticulum fortress (<code>nftables</code> DROP inbound) invisible to IP scanners.</p>
    </article>

    <article class="card">
      <div class="card-head">
        <img src="bifrost://home/icons/bifrost.png" alt="BIFROST" />
        <div>
          <span class="pill holo mono">THE RETICULUM GHOST BROWSER</span>
          <h3 style="margin-top:4px;">BIFROST</h3>
        </div>
      </div>
      <p style="color:var(--muted); font-size:13.5px;">5 MB clean-room Rust browser speaking <code>bifrost://</code> &amp; <code>ironpass://</code> over encrypted X25519 + Ed25519 links — Web2 hard-blocked, RAM-only.</p>
    </article>

    <article class="card">
      <div class="card-head">
        <img src="bifrost://home/icons/ironpass.png" alt="IronPass" />
        <div>
          <span class="pill amber mono">HARDWARE IDENTITY &amp; LEDGER</span>
          <h3 style="margin-top:4px;">IRONPASS</h3>
        </div>
      </div>
      <p style="color:var(--muted); font-size:13.5px;">Hybrid <code>#ip=</code> NFC/QR tags serve public consumer EU ESPR passports while unlocking classified BOMs &amp; hash-chained GS1 EPCIS 2.0 Ledgers in BIFROST.</p>
    </article>
  </section>

  <section class="card">
    <div class="mono" style="font-size:11px; font-weight:800; color:var(--holo);">EXECUTIVE CONTRAST // CIRPASS-2 IN CHROME VS. BIFROST</div>
    <table>
      <thead>
        <tr class="mono">
          <th>Layer</th>
          <th style="color:#fca5a5;">Standard Web2 DPP (Chrome)</th>
          <th style="color:#bef264;">BIFROST (bifrost:// &amp; ironpass://)</th>
        </tr>
      </thead>
      <tbody>
        <tr><td class="mono">Schema</td><td>EU ESPR, W3C VC, GS1 EPCIS 2.0</td><td><strong>100% compatible JSON</strong></td></tr>
        <tr><td class="mono">Exposure</td><td>Public IP + DNS + :443 + Cloud CDN</td><td><strong>0 open ports · 0 local HTTP sockets · 0 DNS</strong></td></tr>
        <tr><td class="mono">Client</td><td>Cookies, SSD cache, extensions, telemetry</td><td><strong>RAM-only incognito · Web2 hard-blocked</strong></td></tr>
        <tr><td class="mono">Verification</td><td>Remote JavaScript from web server</td><td><strong>Local Rust Ed25519 + SHA-256 chain (~30 ms)</strong></td></tr>
        <tr><td class="mono">Resilience</td><td>Heavy web app — dies offline</td><td><strong>~1.7 KB signed JSON — Fiber, Starlink, 5G &amp; LoRa</strong></td></tr>
      </tbody>
    </table>
  </section>
</div>
</body>
</html>
"##;
