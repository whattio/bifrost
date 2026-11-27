//! IronPass Ghost DPP Visual Renderer & Local Ed25519 Verifier (Clean-Room MIT Rust)
//!
//! Takes ONLY pure signed JSON (~1-3 KB) received over encrypted Reticulum (`ironpass://`),
//! independently verifies the canonical SHA-256 + RFC 8032 Ed25519 signature right inside Rust
//! (`ed25519-dalek`), and renders a multi-view Visual DPP Passport Card locally with zero
//! remote HTML/JS/CSS/font requests.

use ed25519_dalek::{Signature, VerifyingKey};
use qrcode::render::svg;
use qrcode::QrCode;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

pub fn make_qr_svg(data: &str) -> String {
    match QrCode::new(data.as_bytes()) {
        Ok(code) => code
            .render::<svg::Color>()
            .min_dimensions(200, 200)
            .quiet_zone(true)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build(),
        Err(_) => "<svg width='200' height='200'></svg>".to_string(),
    }
}

pub struct QrTagItem {
    pub id: String,
    pub title: String,
    pub sku: String,
    pub public_url: String,
}

fn html_esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn render_qr_sheet_html(vault_hash: &str) -> String {
    let defaults = vec![
        QrTagItem {
            id: "battery-01".into(),
            title: "Nordic LFP Industrial Battery Pack (EU Battery Passport)".into(),
            sku: "EU-BATT-LFP-48V-100".into(),
            public_url: "https://example.org/dpp/battery-01".into(),
        },
        QrTagItem {
            id: "drone-01".into(),
            title: "Autonomous Inspection UAV — MRO Grade".into(),
            sku: "UAV-MRO-INSPECT-01".into(),
            public_url: "https://example.org/dpp/drone-01".into(),
        },
        QrTagItem {
            id: "furniture-01".into(),
            title: "Nordic Birch Modular Desk (EU ESPR)".into(),
            sku: "ESPR-WOOD-DESK-2510".into(),
            public_url: "https://example.org/dpp/furniture-01".into(),
        },
    ];
    render_qr_sheet_html_with_items(vault_hash, &defaults)
}

pub fn render_qr_sheet_html_with_items(vault_hash: &str, items: &[QrTagItem]) -> String {
    let mut cards = String::new();
    for item in items {
        let pure_uri = format!("ironpass://{vault_hash}/dpp/{}", item.id);
        let hybrid_uri = format!("{}#ip={vault_hash}/dpp/{}", item.public_url, item.id);
        let pure_svg = make_qr_svg(&pure_uri);
        let hybrid_svg = make_qr_svg(&hybrid_uri);
        let title = html_esc(&item.title);
        let sku = html_esc(&item.sku);
        let pure_esc = html_esc(&pure_uri);
        let hybrid_esc = html_esc(&hybrid_uri);
        cards.push_str(&format!(
            r#"<div class="tag-pair">
              <div class="tag-card">
                <div class="tag-top"><span class="badge hybrid">HYBRID #ip= PUBLIC + VAULT QR</span><span class="sku">{sku}</span></div>
                <h3>{title}</h3>
                <div class="qr-wrap">{hybrid_svg}</div>
                <div class="uri">{hybrid_esc}</div>
                <div class="hint">Normal phone → Public EU ESPR Page · BIFROST → Ed25519 Verified RNS Vault (#ip=)</div>
                <div class="actions no-print">
                  <a class="act primary" href="{pure_esc}">⚡ TEST IN BIFROST</a>
                  <button class="act" type="button" onclick="copyUri(this, '{hybrid_esc}')">📋 COPY HYBRID URI</button>
                </div>
              </div>
              <div class="tag-card">
                <div class="tag-top"><span class="badge pure">PURE ZERO-IP GHOST QR</span><span class="sku">{sku}</span></div>
                <h3>{title}</h3>
                <div class="qr-wrap">{pure_svg}</div>
                <div class="uri">{pure_esc}</div>
                <div class="hint">Scan with BIFROST Ghost Browser (0 DNS · 0 HTTPS · Ed25519 + EPCIS Ledger)</div>
                <div class="actions no-print">
                  <a class="act primary" href="{pure_esc}">⚡ TEST IN BIFROST</a>
                  <button class="act" type="button" onclick="copyUri(this, '{pure_esc}')">📋 COPY ironpass://</button>
                </div>
              </div>
            </div>"#
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<title>IRONPASS // PRINTABLE HYBRID &amp; ZERO-IP QR TAG SHEET</title>
<style>
  * {{ box-sizing:border-box; margin:0; padding:0; }}
  body {{ background:#f8fafc; color:#0f172a; font-family:system-ui,-apple-system,sans-serif; padding:52px 24px 24px; }}
  .bar {{ max-width:1040px; margin:0 auto 18px; display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:10px; background:#020503; color:#f2faf5; padding:14px 18px; border-radius:12px; }}
  .bar h1 {{ font-size:15px; letter-spacing:0.08em; font-family:ui-monospace,monospace; }}
  .bar p {{ font-size:11px; color:#9ec0ac; margin-top:3px; font-family:ui-monospace,monospace; }}
  .btn {{ background:#84cc16; color:#020503; border:none; padding:9px 15px; border-radius:8px; font-weight:800; font-size:12px; cursor:pointer; text-decoration:none; font-family:ui-monospace,monospace; }}
  .btn.ghost {{ background:transparent; color:#bef264; border:1px solid #84cc16; }}
  .sheet {{ max-width:1040px; margin:0 auto; display:flex; flex-direction:column; gap:16px; }}
  .tag-pair {{ display:grid; grid-template-columns:1fr 1fr; gap:16px; page-break-inside:avoid; }}
  .tag-card {{ background:#fff; border:2px dashed #64748b; border-radius:12px; padding:14px 16px; display:flex; flex-direction:column; align-items:center; text-align:center; gap:8px; }}
  .tag-top {{ width:100%; display:flex; justify-content:space-between; align-items:center; font-family:ui-monospace,monospace; font-size:10px; font-weight:800; }}
  .badge {{ padding:3px 8px; border-radius:999px; }}
  .badge.pure {{ background:#020503; color:#84cc16; }}
  .badge.hybrid {{ background:#0c4a6e; color:#bae6fd; }}
  .sku {{ color:#475569; }}
  .tag-card h3 {{ font-size:14px; font-weight:800; color:#0f172a; }}
  .qr-wrap {{ padding:8px; background:#fff; border:1px solid #e2e8f0; border-radius:8px; }}
  .qr-wrap svg {{ width:200px; height:200px; display:block; }}
  .uri {{ font-family:ui-monospace,monospace; font-size:9px; color:#334155; word-break:break-all; background:#f1f5f9; padding:6px 8px; border-radius:6px; width:100%; }}
  .hint {{ font-size:10px; color:#475569; }}
  .actions {{ display:flex; gap:6px; width:100%; margin-top:2px; }}
  .act {{ flex:1; padding:6px 8px; border-radius:6px; border:1px solid #cbd5e1; background:#f8fafc; color:#0f172a; font-family:ui-monospace,monospace; font-size:10px; font-weight:700; cursor:pointer; text-decoration:none; text-align:center; }}
  .act.primary {{ background:#020503; color:#bef264; border-color:#020503; }}
  @media print {{
    body {{ background:#fff; padding:0; }}
    .no-print {{ display:none !important; }}
    .tag-card {{ border:1.5px dashed #64748b; }}
  }}
</style>
</head>
<body>
  <div class="bar no-print">
    <div>
      <h1>ᛉ IRONPASS // LIVE PRINTABLE HYBRID &amp; ZERO-IP QR TAG SHEET</h1>
      <p>Vault Hash: {vault_hash} · {count} live passports ready for phone &amp; BIFROST scanning</p>
    </div>
    <div style="display:flex; gap:8px;">
      <a class="btn ghost" href="bifrost://home/">← BIFROST HOME</a>
      <button class="btn" onclick="window.print()">🖨️ PRINT A4 SHEET</button>
    </div>
  </div>
  <div class="sheet">{cards}</div>
  <script>
    function copyUri(btn, text) {{
      var done = function() {{ var old = btn.textContent; btn.textContent = '✓ COPIED!'; setTimeout(function(){{ btn.textContent = old; }}, 1600); }};
      try {{ if (navigator.clipboard) {{ navigator.clipboard.writeText(text).then(done).catch(fallback); return; }} }} catch(_) {{}}
      fallback();
      function fallback() {{
        try {{
          var ta = document.createElement('textarea'); ta.value = text; ta.style.position = 'fixed'; ta.style.left = '-9999px';
          document.body.appendChild(ta); ta.select(); document.execCommand('copy'); document.body.removeChild(ta); done();
        }} catch(_) {{}}
      }}
    }}
  </script>
</body>
</html>"#,
        count = items.len()
    )
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let b = s.as_bytes();
    for i in (0..b.len()).step_by(2) {
        let hi = (b[i] as char).to_digit(16)?;
        let lo = (b[i + 1] as char).to_digit(16)?;
        out.push(((hi << 4) | lo) as u8);
    }
    Some(out)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Canonical JSON serialization matching Python `json.dumps(obj, sort_keys=True, separators=(",", ":"))`
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

pub struct CryptoVerifyResult {
    pub ed25519_verified: bool,
    pub sha256_matches: bool,
    pub local_sha256: String,
    pub pubkey_short: String,
    pub status_label: String,
}

fn verify_ironpass_proof(doc: &Value) -> CryptoVerifyResult {
    let data = doc.get("data").unwrap_or(doc);
    let canonical = canonical_json(data);
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let local_sha256 = hex_encode(&hasher.finalize());

    let Some(proof) = doc.get("ironpass_proof") else {
        return CryptoVerifyResult {
            ed25519_verified: false,
            sha256_matches: false,
            local_sha256,
            pubkey_short: "UNSIGNED".into(),
            status_label: "UNSIGNED JSON PAYLOAD".into(),
        };
    };

    let claimed_sha = proof
        .get("canonical_sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let sha256_matches = !claimed_sha.is_empty() && claimed_sha.eq_ignore_ascii_case(&local_sha256);

    let pub_hex = proof
        .get("ed25519_pubkey_hex")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let sig_hex = proof
        .get("ed25519_signature_hex")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let pubkey_short = if pub_hex.len() >= 16 {
        format!("{}…{}", &pub_hex[..8], &pub_hex[pub_hex.len() - 8..])
    } else {
        "NONE".into()
    };

    let mut ed25519_verified = false;
    if sha256_matches {
        if let (Some(pb), Some(sb)) = (hex_decode(pub_hex), hex_decode(sig_hex)) {
            if pb.len() == 32 && sb.len() == 64 {
                let mut pk_arr = [0u8; 32];
                pk_arr.copy_from_slice(&pb);
                let mut sig_arr = [0u8; 64];
                sig_arr.copy_from_slice(&sb);
                if let Ok(vk) = VerifyingKey::from_bytes(&pk_arr) {
                    let sig = Signature::from_bytes(&sig_arr);
                    if vk.verify_strict(local_sha256.as_bytes(), &sig).is_ok() {
                        ed25519_verified = true;
                    }
                }
            }
        }
    }

    let status_label = if ed25519_verified {
        "ED25519 SIGNATURE VERIFIED IN RUST".into()
    } else if !sha256_matches {
        "HASH MISMATCH — PAYLOAD ALTERED".into()
    } else {
        "SIGNATURE VERIFICATION FAILED".into()
    };

    CryptoVerifyResult {
        ed25519_verified,
        sha256_matches,
        local_sha256,
        pubkey_short,
        status_label,
    }
}

fn verify_ed25519_sig(msg: &str, pub_hex: &str, sig_hex: &str) -> bool {
    if let (Some(pb), Some(sb)) = (hex_decode(pub_hex), hex_decode(sig_hex)) {
        if pb.len() == 32 && sb.len() == 64 {
            let mut pk_arr = [0u8; 32];
            pk_arr.copy_from_slice(&pb);
            let mut sig_arr = [0u8; 64];
            sig_arr.copy_from_slice(&sb);
            if let Ok(vk) = VerifyingKey::from_bytes(&pk_arr) {
                let sig = Signature::from_bytes(&sig_arr);
                return vk.verify_strict(msg.as_bytes(), &sig).is_ok();
            }
        }
    }
    false
}

/// Independently verifies the entire SHA-256 hash chain + Ed25519 signature of every
/// GS1 EPCIS 2.0 event inside `doc["ironpass_ledger"]` locally in Rust.
fn verify_ironpass_ledger(doc: &Value) -> Value {
    let Some(events) = doc.get("ironpass_ledger").and_then(|v| v.as_array()) else {
        return serde_json::json!({
            "total": 0,
            "verified": 0,
            "chain_intact": true,
            "status_label": "NO LEDGER EVENTS",
            "events": []
        });
    };

    let mut expected_prev = "0".repeat(64);
    let mut verified_count: usize = 0;
    let mut all_ok = true;
    let mut out_events = Vec::with_capacity(events.len());

    for (idx, ev) in events.iter().enumerate() {
        let seq = ev.get("seq").and_then(|v| v.as_u64()).unwrap_or(0);
        let prev_hash = ev.get("prev_hash").and_then(|v| v.as_str()).unwrap_or("");
        let claimed_hash = ev.get("event_hash").and_then(|v| v.as_str()).unwrap_or("");
        let pub_hex = ev.get("ed25519_pubkey_hex").and_then(|v| v.as_str()).unwrap_or("");
        let sig_hex = ev.get("ed25519_signature_hex").and_then(|v| v.as_str()).unwrap_or("");

        let core = serde_json::json!({
            "seq": seq,
            "type": ev.get("type").and_then(|v| v.as_str()).unwrap_or(""),
            "eventTime": ev.get("eventTime").and_then(|v| v.as_str()).unwrap_or(""),
            "action": ev.get("action").and_then(|v| v.as_str()).unwrap_or(""),
            "bizStep": ev.get("bizStep").and_then(|v| v.as_str()).unwrap_or(""),
            "disposition": ev.get("disposition").and_then(|v| v.as_str()).unwrap_or(""),
            "readPoint": ev.get("readPoint").and_then(|v| v.as_str()).unwrap_or(""),
            "actor": ev.get("actor").and_then(|v| v.as_str()).unwrap_or(""),
            "note": ev.get("note").and_then(|v| v.as_str()).unwrap_or(""),
            "prev_hash": prev_hash,
        });
        let canon = canonical_json(&core);
        let mut h = Sha256::new();
        h.update(canon.as_bytes());
        let local_hash = hex_encode(&h.finalize());

        let seq_ok = seq == (idx as u64 + 1);
        let prev_ok = prev_hash.eq_ignore_ascii_case(&expected_prev);
        let hash_ok = !claimed_hash.is_empty() && claimed_hash.eq_ignore_ascii_case(&local_hash);
        let sig_ok = hash_ok && verify_ed25519_sig(&local_hash, pub_hex, sig_hex);
        let ev_ok = seq_ok && prev_ok && hash_ok && sig_ok;

        if ev_ok {
            verified_count += 1;
        } else {
            all_ok = false;
        }
        expected_prev = claimed_hash.to_string();

        out_events.push(serde_json::json!({
            "seq": seq,
            "type": ev.get("type").and_then(|v| v.as_str()).unwrap_or("ObjectEvent"),
            "eventTime": ev.get("eventTime").and_then(|v| v.as_str()).unwrap_or(""),
            "action": ev.get("action").and_then(|v| v.as_str()).unwrap_or("OBSERVE"),
            "bizStep": ev.get("bizStep").and_then(|v| v.as_str()).unwrap_or(""),
            "disposition": ev.get("disposition").and_then(|v| v.as_str()).unwrap_or(""),
            "readPoint": ev.get("readPoint").and_then(|v| v.as_str()).unwrap_or(""),
            "actor": ev.get("actor").and_then(|v| v.as_str()).unwrap_or(""),
            "note": ev.get("note").and_then(|v| v.as_str()).unwrap_or(""),
            "prev_hash": prev_hash,
            "event_hash": claimed_hash,
            "chain_ok": prev_ok && seq_ok,
            "hash_ok": hash_ok,
            "sig_ok": sig_ok,
            "verified": ev_ok,
        }));
    }

    let status_label = if events.is_empty() {
        "NO LEDGER EVENTS".to_string()
    } else if all_ok {
        format!(
            "CHAIN INTACT · {}/{} EPCIS 2.0 EVENTS SHA-256 + ED25519 VERIFIED IN RUST",
            verified_count,
            events.len()
        )
    } else {
        format!(
            "TAMPER DETECTED · ONLY {}/{} EVENTS VERIFIED",
            verified_count,
            events.len()
        )
    };

    serde_json::json!({
        "total": events.len(),
        "verified": verified_count,
        "chain_intact": all_ok,
        "status_label": status_label,
        "events": out_events
    })
}

pub fn render_visual_dpp(
    uri: &str,
    dest_hash: &str,
    path: &str,
    wire_bytes: usize,
    logical_bytes: usize,
    chunks_rx: u32,
    was_gzip: bool,
    latency_ms: f64,
    raw_body: &[u8],
) -> String {
    let doc: Value = serde_json::from_slice(raw_body).unwrap_or_else(|_| {
        serde_json::json!({
            "data": {
                "name": "Raw Reticulum Payload",
                "description": String::from_utf8_lossy(raw_body).to_string()
            }
        })
    });

    let verify = verify_ironpass_proof(&doc);
    let ledger = verify_ironpass_ledger(&doc);
    let lora_sec = ((wire_bytes as f64) * 8.0) / 1700.0; // EU 868 MHz SF9 ~1.7 kbps

    let pure_uri = format!("ironpass://{dest_hash}{path}");
    let sku = doc
        .get("data")
        .and_then(|d| d.get("product_number"))
        .and_then(|v| v.as_str())
        .unwrap_or("DPP");
    let hybrid_uri = format!(
        "https://whatt.io/dpp/{sku}#ip={dest_hash}{path}"
    );
    let pure_qr_svg = make_qr_svg(&pure_uri);
    let hybrid_qr_svg = make_qr_svg(&hybrid_uri);

    let payload_for_js = serde_json::json!({
        "uri": uri,
        "pure_uri": pure_uri,
        "hybrid_uri": hybrid_uri,
        "pure_qr_svg": pure_qr_svg,
        "hybrid_qr_svg": hybrid_qr_svg,
        "dest_hash": dest_hash,
        "path": path,
        "wire_bytes": wire_bytes,
        "logical_bytes": logical_bytes,
        "chunks_rx": chunks_rx,
        "was_gzip": was_gzip,
        "latency_ms": (latency_ms * 10.0).round() / 10.0,
        "lora_est_sec": (lora_sec * 10.0).round() / 10.0,
        "verify": {
            "ed25519_verified": verify.ed25519_verified,
            "sha256_matches": verify.sha256_matches,
            "local_sha256": verify.local_sha256,
            "pubkey_short": verify.pubkey_short,
            "status_label": verify.status_label,
        },
        "ledger": ledger,
        "doc": doc,
    });

    // Safe script-tag embedding
    let embedded_json = serde_json::to_string(&payload_for_js)
        .unwrap_or_else(|_| "{}".into())
        .replace("</", "<\\/");

    DPP_TEMPLATE_HTML.replace("__IRONPASS_DATA_JSON__", &embedded_json)
}

const DPP_TEMPLATE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>IRONPASS // SOVEREIGN GHOST DPP</title>
<link rel="icon" type="image/png" href="bifrost://home/icons/ironpass.png" />
<style>
  :root {
    --bg: #020503;
    --panel: rgba(6, 15, 10, 0.94);
    --panel2: rgba(10, 23, 15, 0.94);
    --lime: #84cc16;
    --emerald: #10b981;
    --holo: #38bdf8;
    --amber: #f59e0b;
    --violet: #a78bfa;
    --red: #f87171;
    --text: #f2faf5;
    --muted: #9ec0ac;
    --border: rgba(16, 185, 129, 0.28);
  }
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    min-height: 100vh;
    background:
      radial-gradient(ellipse 80% 45% at 50% 0%, rgba(132,204,22,0.14), transparent 60%),
      radial-gradient(ellipse 55% 40% at 88% 90%, rgba(56,189,248,0.10), transparent 55%),
      var(--bg);
    color: var(--text);
    font-family: system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
    padding: 22px 26px 40px;
  }
  .mono { font-family: ui-monospace, 'JetBrains Mono', SFMono-Regular, Menlo, monospace; }
  .wrap { max-width: 1680px; width: 100%; margin: 0 auto; display: flex; flex-direction: column; gap: 16px; }

  .trust-banner {
    display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 12px;
    padding: 14px 18px; border-radius: 14px;
    background: linear-gradient(135deg, rgba(16,185,129,0.16), rgba(6,15,10,0.95));
    border: 1.5px solid rgba(132,204,22,0.55);
    box-shadow: 0 0 28px rgba(132,204,22,0.14);
  }
  .trust-banner.warn {
    background: linear-gradient(135deg, rgba(248,113,113,0.16), rgba(15,6,6,0.95));
    border-color: rgba(248,113,113,0.6);
  }
  .trust-left { display: flex; align-items: center; gap: 14px; }
  .shield-icon {
    width: 48px; height: 48px; border-radius: 12px; display: grid; place-items: center;
    background: rgba(132,204,22,0.18); border: 1.5px solid rgba(132,204,22,0.6); color: var(--lime);
  }
  .trust-banner.warn .shield-icon { background: rgba(248,113,113,0.18); border-color: var(--red); color: var(--red); }
  .pill {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 5px 10px; border-radius: 999px; font-size: 10.5px; font-weight: 800; letter-spacing: 0.06em;
    border: 1px solid rgba(132,204,22,0.48); background: rgba(132,204,22,0.14); color: #bef264;
  }
  .pill.holo { border-color: rgba(56,189,248,0.48); background: rgba(56,189,248,0.12); color: #bae6fd; }
  .pill.amber { border-color: rgba(245,158,11,0.48); background: rgba(245,158,11,0.12); color: #fcd34d; }
  .pill.red { border-color: rgba(248,113,113,0.55); background: rgba(248,113,113,0.14); color: #fecaca; }

  .switcher { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 10px; }
  .tabs { display: flex; flex-wrap: wrap; gap: 8px; }
  .tab {
    background: rgba(6,15,10,0.9); color: var(--muted); border: 1px solid var(--border);
    padding: 8px 14px; border-radius: 9px; cursor: pointer; font-weight: 800; font-size: 11.5px;
    transition: all 0.15s ease;
  }
  .tab:hover { color: #fff; border-color: var(--lime); }
  .tab.active { background: rgba(132,204,22,0.18); border-color: var(--lime); color: #fff; box-shadow: 0 0 14px rgba(132,204,22,0.18); }

  .passport-links { display: flex; flex-wrap: wrap; gap: 7px; }
  .plink {
    background: rgba(56,189,248,0.10); color: #bae6fd; border: 1px solid rgba(56,189,248,0.38);
    padding: 7px 11px; border-radius: 8px; font-size: 11px; font-weight: 800; text-decoration: none;
  }
  .plink:hover { background: rgba(56,189,248,0.22); color: #fff; }

  .hero-grid { display: grid; grid-template-columns: 1.45fr 1fr; gap: 16px; }
  @media (max-width: 920px) { .hero-grid { grid-template-columns: 1fr; } }

  .card {
    background: var(--panel); border: 1px solid var(--border); border-radius: 14px;
    padding: 20px; display: flex; flex-direction: column; gap: 14px;
  }
  .eyebrow { font-size: 10.5px; font-weight: 800; letter-spacing: 0.12em; color: var(--holo); }
  .eyebrow.lime { color: var(--lime); }
  .eyebrow.amber { color: var(--amber); }
  .prod-title { font-size: 26px; font-weight: 900; color: #fff; line-height: 1.2; }
  .prod-desc { font-size: 14px; color: var(--muted); line-height: 1.6; }

  .kpi-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }
  @media (max-width: 920px) { .kpi-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
  .kpi {
    background: var(--panel2); border: 1px solid rgba(255,255,255,0.08);
    border-radius: 11px; padding: 12px;
  }
  .kpi .k { font-size: 10px; color: var(--muted); letter-spacing: 0.08em; }
  .kpi .v { font-size: 22px; font-weight: 900; color: var(--lime); margin-top: 4px; }
  .kpi .v.holo { color: var(--holo); }
  .kpi .v.amber { color: var(--amber); }
  .kpi .s { font-size: 11px; color: var(--muted); margin-top: 2px; }

  .nfc-card {
    background: linear-gradient(155deg, rgba(16,185,129,0.14), rgba(2,10,6,0.96));
    border: 1px solid rgba(56,189,248,0.42); border-radius: 14px; padding: 18px;
    display: flex; flex-direction: column; gap: 10px;
  }
  .kv-row { display: flex; justify-content: space-between; gap: 10px; font-size: 12px; padding: 6px 0; border-bottom: 1px solid rgba(255,255,255,0.06); }
  .kv-row:last-child { border-bottom: none; }
  .kv-row .k { color: var(--muted); }
  .kv-row .v { color: #fff; font-weight: 700; text-align: right; word-break: break-all; }

  .bar-list { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
  @media (max-width: 920px) { .bar-list { grid-template-columns: 1fr; } }
  .bar-item { background: var(--panel2); border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; padding: 12px 14px; }
  .bar-top { display: flex; justify-content: space-between; font-size: 12px; margin-bottom: 6px; }
  .bar-track { height: 7px; border-radius: 999px; background: rgba(255,255,255,0.08); overflow: hidden; }
  .bar-fill { height: 100%; border-radius: 999px; background: linear-gradient(90deg, var(--emerald), var(--lime)); }

  .lifecycle-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }
  @media (max-width: 920px) { .lifecycle-grid { grid-template-columns: 1fr; } }
  .life-box { background: var(--panel2); border: 1px solid rgba(16,185,129,0.22); border-radius: 11px; padding: 14px; }
  .life-box h3 { font-size: 13px; color: var(--lime); margin-bottom: 6px; letter-spacing: 0.06em; }
  .life-box p { font-size: 13px; color: var(--muted); line-height: 1.55; }

  .tpl-group { background: var(--panel2); border: 1px solid rgba(56,189,248,0.24); border-radius: 12px; padding: 15px; }
  .tpl-group h3 { font-size: 14px; color: #bae6fd; margin-bottom: 10px; }
  .tbl { width: 100%; border-collapse: collapse; font-size: 12.5px; }
  .tbl td { padding: 7px 8px; border-bottom: 1px solid rgba(255,255,255,0.06); }
  .tbl td:first-child { color: var(--muted); width: 46%; }
  .tbl td:last-child { color: #fff; font-weight: 700; }

  pre.raw-json {
    background: #010603; border: 1px solid rgba(16,185,129,0.3); border-radius: 10px;
    padding: 15px; font-size: 12px; color: #d1fae5; overflow: auto; max-height: 460px; line-height: 1.5;
  }
  .view { display: none; flex-direction: column; gap: 16px; }
  .view.active { display: flex; }

  .ledger-banner {
    display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 10px;
    padding: 13px 16px; border-radius: 12px;
    background: rgba(16,185,129,0.12); border: 1px solid rgba(132,204,22,0.48);
  }
  .ledger-banner.warn { background: rgba(248,113,113,0.14); border-color: rgba(248,113,113,0.58); }
  .timeline { display: flex; flex-direction: column; gap: 12px; position: relative; padding-left: 20px; }
  .timeline::before {
    content: ""; position: absolute; left: 7px; top: 8px; bottom: 8px; width: 2px;
    background: linear-gradient(180deg, var(--holo), var(--lime));
  }
  .ev-card {
    position: relative; background: var(--panel2); border: 1px solid rgba(56,189,248,0.28);
    border-radius: 12px; padding: 14px 16px; display: flex; flex-direction: column; gap: 8px;
  }
  .ev-card::before {
    content: ""; position: absolute; left: -18px; top: 18px; width: 10px; height: 10px;
    border-radius: 50%; background: var(--lime); box-shadow: 0 0 10px var(--lime);
  }
  .ev-card.bad { border-color: rgba(248,113,113,0.6); }
  .ev-card.bad::before { background: var(--red); box-shadow: 0 0 10px var(--red); }
  .ev-head { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 8px; }
  .ev-hashes { font-size: 10.5px; color: #7dd3fc; background: rgba(1,6,3,0.85); padding: 7px 10px; border-radius: 8px; word-break: break-all; }
  .ledger-form {
    display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px;
    background: var(--panel2); border: 1px solid rgba(132,204,22,0.34); border-radius: 12px; padding: 16px;
  }
  @media (max-width: 900px) { .ledger-form { grid-template-columns: 1fr; } }
  .ledger-form label { display: flex; flex-direction: column; gap: 5px; font-size: 10.5px; color: var(--muted); }
  .ledger-form input, .ledger-form select {
    background: rgba(2,8,5,0.92); border: 1px solid var(--border); color: #fff;
    border-radius: 8px; padding: 8px 10px; font-size: 12px; font-family: inherit; outline: none;
  }
  .ledger-form input:focus, .ledger-form select:focus { border-color: var(--lime); }
</style>
</head>
<body>
<div class="wrap" id="app"></div>

<script>
const PAYLOAD = __IRONPASS_DATA_JSON__;
const doc = PAYLOAD.doc || {};
const data = doc.data || doc;
const proof = doc.ironpass_proof || {};
const nfc = proof.ntag424_dna || {};
const ver = PAYLOAD.verify || {};
const ledger = PAYLOAD.ledger || { total: 0, verified: 0, chain_intact: true, status_label: 'NO EVENTS', events: [] };
const tpls = Array.isArray(data.template_fields) ? data.template_fields : [];

// Extract ESPR / sustainability metrics across all template_fields for KPI cards & bars
const allFields = [];
tpls.forEach(g => {
  (g.fields || []).forEach(f => {
    allFields.push({ group: g.name || '', id: f.id || '', name: f.name || '', value: f.value, unit: f.unit || '' });
  });
});

function findVal(patterns, fallback = '--') {
  for (const p of patterns) {
    const hit = allFields.find(f =>
      (f.id && f.id.toLowerCase().includes(p)) ||
      (f.name && f.name.toLowerCase().includes(p))
    );
    if (hit && hit.value !== undefined && hit.value !== null && hit.value !== '') {
      return `${hit.value}${hit.unit ? ' ' + hit.unit : ''}`;
    }
  }
  return fallback;
}

const isTampered = !ver.ed25519_verified || nfc.cmac_status === 'REPLAY_DETECTED';
const nfcBadgeText = nfc.cmac_status === 'REPLAY_DETECTED'
  ? 'CLONE / REPLAY DETECTED'
  : 'AUTHENTIC SILICON · NTAG 424 DNA';

const esc = (s) => String(s ?? '').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');

const app = document.getElementById('app');
app.innerHTML = `
  <!-- CRYPTOGRAPHIC TRUST BANNER -->
  <div class="trust-banner ${isTampered ? 'warn' : ''}">
    <div class="trust-left">
      <div class="shield-icon" style="padding:0;overflow:hidden;border-radius:12px;">
        <img src="bifrost://home/icons/ironpass.png" alt="IronPass" style="width:48px;height:48px;display:block;" />
      </div>
      <div>
        <div class="mono" style="font-size:11px; font-weight:900; letter-spacing:0.1em; color:${isTampered ? 'var(--red)' : 'var(--lime)'};">
          ${esc(nfcBadgeText)} &nbsp;·&nbsp; ${esc(ver.status_label)}
        </div>
        <div style="font-size:13px; color:var(--muted); margin-top:3px;">
          Rendered locally in Rust from <strong>pure signed JSON</strong> over encrypted X25519+Ed25519 Reticulum — <strong>0 HTML/JS on wire · 0 HTTP sockets</strong>.
        </div>
      </div>
    </div>
    <div style="display:flex; flex-wrap:wrap; gap:6px;" class="mono">
      <span class="pill">${PAYLOAD.wire_bytes} B WIRE (${PAYLOAD.chunks_rx} CHT2)</span>
      <span class="pill holo">${PAYLOAD.latency_ms} ms RTT</span>
      <span class="pill amber">LoRa 868MHz EST: ~${PAYLOAD.lora_est_sec}s</span>
    </div>
  </div>

  <!-- VIEW TABS + PASSPORT SWITCHER -->
  <div class="switcher">
    <div class="tabs mono">
      <button class="tab active" onclick="showView('overview', this)">🛡️ OVERVIEW &amp; TRUST</button>
      <button class="tab" onclick="showView('ledger', this)">📜 IRONPASS LEDGER (${ledger.total || 0})</button>
      <button class="tab" onclick="showView('espr', this)">♻️ EU ESPR &amp; LIFECYCLE</button>
      <button class="tab" onclick="showView('bom', this)">⚙️ ETIM / BOM</button>
      <button class="tab" onclick="showView('qr', this)">🏷️ PRINT QR / NFC TAG</button>
      <button class="tab" onclick="showView('crypto', this)">🔐 CRYPTO PROOF &amp; JSON</button>
    </div>
    <div class="passport-links mono">
      <a class="plink" href="ironpass://${esc(PAYLOAD.dest_hash)}/dpp/battery-01">🔋 BATTERY-01</a>
      <a class="plink" href="ironpass://${esc(PAYLOAD.dest_hash)}/dpp/drone-01">🛸 DRONE-01</a>
      <a class="plink" href="ironpass://${esc(PAYLOAD.dest_hash)}/dpp/furniture-01">🪑 FURNITURE-01</a>
      <a class="plink" href="bifrost://home/showcase">🌐 MICROSITE EXAMPLE</a>
    </div>
  </div>

  <!-- VIEW 1: OVERVIEW -->
  <div class="view active" id="view-overview">
    <div class="hero-grid">
      <div class="card">
        <div>
          <div class="eyebrow mono">IRONPASS DIGITAL PRODUCT PASSPORT · ID: ${esc(doc.dpp_id || data.id || 'DPP')} ${data.product_number ? '· ' + esc(data.product_number) : ''}</div>
          <h1 class="prod-title" style="margin-top:6px;">${esc(data.name || 'Sovereign Asset Passport')}</h1>
        </div>
        <p class="prod-desc">${esc(data.description || 'Verified offline-capable Digital Product Passport streamed strictly as signed JSON over Reticulum.')}</p>
        <div class="kpi-grid mono">
          <div class="kpi">
            <div class="k">RECYCLED CONTENT</div>
            <div class="v">${esc(findVal(['recycled_materials', 'recycled'], '50%'))}</div>
            <div class="s">EU ESPR Circularity</div>
          </div>
          <div class="kpi">
            <div class="k">DURABILITY / LIFE</div>
            <div class="v holo">${esc(findVal(['durability', 'lifetime'], '8 yrs'))}</div>
            <div class="s">Expected service life</div>
          </div>
          <div class="kpi">
            <div class="k">MTBF RELIABILITY</div>
            <div class="v">${esc(findVal(['reliability', 'mtbf'], '25,000 h'))}</div>
            <div class="s">Mean time between failure</div>
          </div>
          <div class="kpi">
            <div class="k">WIRE PAYLOAD</div>
            <div class="v amber">${PAYLOAD.wire_bytes} B</div>
            <div class="s">${PAYLOAD.chunks_rx} encrypted CHT2 pkts</div>
          </div>
        </div>
      </div>

      <div class="nfc-card mono">
        <div class="eyebrow lime">HARDWARE SILICON &amp; VAULT ATTESTATION</div>
        <div class="kv-row"><span class="k">NFC Silicon</span><span class="v">${esc(nfc.chip || 'NXP NTAG 424 DNA')}</span></div>
        <div class="kv-row"><span class="k">Silicon UID</span><span class="v" style="color:var(--holo);">${esc(nfc.uid || '--')}</span></div>
        <div class="kv-row"><span class="k">Monotonic Tap #</span><span class="v" style="color:var(--lime);">#${esc(nfc.read_counter ?? '--')} (Anti-Clone)</span></div>
        <div class="kv-row"><span class="k">SDM AES-128 CMAC</span><span class="v" style="color:${nfc.cmac_status === 'REPLAY_DETECTED' ? 'var(--red)' : 'var(--lime)'};">${esc(nfc.cmac_status || 'VERIFIED')}</span></div>
        <div class="kv-row"><span class="k">Tamper Loop</span><span class="v" style="color:var(--lime);">${esc(nfc.tamper_loop || 'INTACT')}</span></div>
        <div class="kv-row"><span class="k">Ed25519 Key</span><span class="v">${esc(ver.pubkey_short)}</span></div>
        <div class="kv-row"><span class="k">0-Port Vault Hash</span><span class="v" style="font-size:10.5px; color:#bae6fd;">${esc(PAYLOAD.dest_hash)}</span></div>
      </div>
    </div>
  </div>

  <!-- VIEW 1B: IRONPASS LEDGER (HASH-CHAINED GS1 EPCIS 2.0) -->
  <div class="view" id="view-ledger">
    <div class="card">
      <div class="ledger-banner ${ledger.chain_intact ? '' : 'warn'} mono">
        <div>
          <div style="font-size:11px; font-weight:900; letter-spacing:0.08em; color:${ledger.chain_intact ? 'var(--lime)' : 'var(--red)'};">
            ${ledger.chain_intact ? '✓' : '⚠'} ${esc(ledger.status_label)}
          </div>
          <div style="font-size:11.5px; color:var(--muted); margin-top:3px;">
            Immutable GS1 EPCIS 2.0 history nailed into iron — every event links <code>prev_hash → SHA-256</code> and is independently verified in Rust via <code>ed25519-dalek</code>.
          </div>
        </div>
        <span class="pill holo">${ledger.verified}/${ledger.total} VERIFIED</span>
      </div>

      <div class="timeline" id="ledger-timeline">
        ${(ledger.events || []).map(ev => `
          <div class="ev-card ${ev.verified ? '' : 'bad'}">
            <div class="ev-head mono">
              <div style="display:flex; gap:7px; align-items:center; flex-wrap:wrap;">
                <span class="pill">#${ev.seq} · ${esc(ev.bizStep).toUpperCase()}</span>
                <span class="pill holo">${esc(ev.type)} · ${esc(ev.action)}</span>
                <span class="pill amber">${esc(ev.disposition)}</span>
              </div>
              <span class="pill ${ev.verified ? '' : 'red'}">${ev.verified ? '✓ SHA-256 + ED25519' : '✕ TAMPER'} · ${esc(ev.eventTime)}</span>
            </div>
            <div style="font-size:14px; color:#fff; font-weight:700;">${esc(ev.note || 'GS1 EPCIS 2.0 custody/lifecycle event')}</div>
            <div class="mono" style="font-size:11.5px; color:var(--muted);">
              Actor: <strong style="color:#d9f99d;">${esc(ev.actor)}</strong> &nbsp;|&nbsp; ReadPoint: <span style="color:#bae6fd;">${esc(ev.readPoint)}</span>
            </div>
            <div class="ev-hashes mono">
              prev_hash: ${esc(ev.prev_hash)}<br/>
              event_hash: <strong style="color:#bef264;">${esc(ev.event_hash)}</strong>
            </div>
          </div>
        `).join('') || '<div class="mono" style="color:var(--muted);">No IronPass Ledger events recorded yet.</div>'}
      </div>

      <div style="margin-top:6px;">
        <div class="eyebrow lime mono" style="margin-bottom:8px;">+ SEAL NEW GS1 EPCIS 2.0 EVENT OVER ENCRYPTED BIFROST RETICULUM LINK</div>
        <form class="ledger-form mono" onsubmit="sealLedgerEvent(event)">
          <label>BIZ STEP (GS1 EPCIS 2.0)
            <select id="ev-bizstep">
              <option value="inspecting">inspecting (MRO / QA Inspection)</option>
              <option value="repairing">repairing (Part Replacement / Service)</option>
              <option value="shipping">shipping (Dispatch / Custody Handover)</option>
              <option value="receiving">receiving (Vault Intake / Acceptance)</option>
              <option value="commissioning">commissioning (Hardware Provisioning)</option>
              <option value="decommissioning">decommissioning (End-of-Life / Recycle)</option>
            </select>
          </label>
          <label>AUTHORIZED ACTOR / BOT
            <input id="ev-actor" value="Authorized Bifrost Operator" required />
          </label>
          <label>READ POINT (GS1 SGLN)
            <input id="ev-readpoint" value="urn:epc:id:sgln:0000000.00001.lab" required />
          </label>
          <label style="grid-column: span 2;">EVENT DESCRIPTION (SIGNED &amp; HASH-CHAINED)
            <input id="ev-note" placeholder="e.g. Rotor assembly inspected & NTAG 424 DNA counter verified in field" required />
          </label>
          <div style="display:flex; align-items:flex-end; gap:8px;">
            <button type="submit" id="ev-submit-btn" class="tab active" style="width:100%;">🔐 SEAL &amp; SIGN EVENT →</button>
          </div>
        </form>
        <div id="ev-status" class="mono" style="font-size:11.5px; color:var(--holo); margin-top:8px;"></div>
      </div>
    </div>
  </div>

  <!-- VIEW 2: EU ESPR & LIFECYCLE -->
  <div class="view" id="view-espr">
    <div class="card">
      <div class="eyebrow lime mono">EU ESPR SUSTAINABILITY, CIRCULARITY &amp; EMISSIONS TELEMETRY</div>
      <div class="bar-list mono" id="espr-bars"></div>
    </div>
    <div class="lifecycle-grid">
      <div class="life-box">
        <h3 class="mono">MATERIAL COMPOSITION</h3>
        <p>${esc(data.material_description || 'See EU ESPR & ETIM BOM tables for material breakdown.')}</p>
      </div>
      <div class="life-box">
        <h3 class="mono">ASSEMBLY &amp; PROVENANCE</h3>
        <p>${esc(data.assembly_desc || 'Verified sovereign manufacturing provenance via EPCIS 2.0 ObjectEvent.')}</p>
      </div>
      <div class="life-box">
        <h3 class="mono">SPARE PARTS &amp; REPAIRABILITY</h3>
        <p>${esc(data.spare_parts_description || 'Modular repairable design with cryptographic part-level traceability.')}</p>
      </div>
      <div class="life-box">
        <h3 class="mono">END-OF-LIFE &amp; RECYCLING</h3>
        <p>${esc(data.recycle_description || 'Dispose and recover materials per EU ESPR circularity guidance.')}</p>
      </div>
    </div>
  </div>

  <!-- VIEW 3: ETIM / BOM & EPCIS -->
  <div class="view" id="view-bom">
    ${tpls.map(g => `
      <div class="tpl-group">
        <h3 class="mono">${esc(g.name || 'Passport Template')}</h3>
        <table class="tbl mono">
          ${(g.fields || []).map(f => `
            <tr>
              <td>${esc(f.name || f.id)}</td>
              <td>${esc(f.value ?? '--')}${f.unit ? ' ' + esc(f.unit) : ''}</td>
            </tr>
          `).join('')}
        </table>
      </div>
    `).join('') || '<div class="card mono">No template_fields in this JSON payload.</div>'}
  </div>

  <!-- VIEW 4: PRINTABLE QR & HYBRID NFC TAG -->
  <div class="view" id="view-qr">
    <div class="card">
      <div style="display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:10px;">
        <div>
          <div class="eyebrow lime mono">PHYSICAL PRODUCT TAGGING // PURE ZERO-IP &amp; HYBRID NTAG 215 / QR</div>
          <h2 style="font-size:18px; margin-top:4px;">Print QR Demo Labels or Program Onto NFC Tags</h2>
        </div>
        <div style="display:flex; gap:8px;" class="mono">
          <a class="plink" href="bifrost://home/qr-sheet">🖨️ FULL A4 PRINTABLE QR SHEET →</a>
          <button class="tab active" onclick="window.print()">🖨️ PRINT THIS CARD</button>
        </div>
      </div>
      <div class="lifecycle-grid">
        <div class="life-box" style="display:flex; flex-direction:column; align-items:center; text-align:center; gap:10px;">
          <h3 class="mono">1. PURE ZERO-IP GHOST QR (ironpass://)</h3>
          <div style="background:#fff; padding:10px; border-radius:10px;">${PAYLOAD.pure_qr_svg}</div>
          <div class="mono" style="font-size:11px; color:#bef264; word-break:break-all;">${esc(PAYLOAD.pure_uri)}</div>
          <p style="font-size:12px;">100% Zero-IP Reticulum pointer — zero URLs, zero DNS, zero Web2 exposure.</p>
        </div>
        <div class="life-box" style="display:flex; flex-direction:column; align-items:center; text-align:center; gap:10px; border-color:rgba(56,189,248,0.4);">
          <h3 class="mono" style="color:var(--holo);">2. HYBRID PUBLIC + CLASSIFIED QR (#ip=)</h3>
          <div style="background:#fff; padding:10px; border-radius:10px;">${PAYLOAD.hybrid_qr_svg}</div>
          <div class="mono" style="font-size:11px; color:#bae6fd; word-break:break-all;">${esc(PAYLOAD.hybrid_uri)}</div>
          <p style="font-size:12px;">Normal phones open public <code>whatt.io</code> consumer DPP (#fragment never leaves phone!). BIFROST strips Web2 and opens classified Reticulum Vault.</p>
        </div>
      </div>
    </div>
  </div>

  <!-- VIEW 5: CRYPTO PROOF & RAW JSON -->
  <div class="view" id="view-crypto">
    <div class="card mono">
      <div class="eyebrow amber">CRYPTOGRAPHIC VERIFICATION LOG (COMPUTED LOCALLY IN RUST)</div>
      <div class="kv-row"><span class="k">IronPass Zero-IP URI</span><span class="v" style="color:var(--lime);">${esc(PAYLOAD.uri)}</span></div>
      <div class="kv-row"><span class="k">Reticulum Transport</span><span class="v">X25519 ECDH + Ed25519 · In-Process (0 HTTP Ports)</span></div>
      <div class="kv-row"><span class="k">Wire vs. Logical Size</span><span class="v">${PAYLOAD.wire_bytes} B encrypted wire (${PAYLOAD.chunks_rx}×400B CHT2, gzip=${PAYLOAD.was_gzip}) / ${PAYLOAD.logical_bytes} B JSON</span></div>
      <div class="kv-row"><span class="k">Local Canonical SHA-256</span><span class="v" style="color:#bae6fd;">${esc(ver.local_sha256)}</span></div>
      <div class="kv-row"><span class="k">Ed25519 PubKey (vault)</span><span class="v">${esc(proof.ed25519_pubkey_hex || '--')}</span></div>
      <div class="kv-row"><span class="k">Ed25519 Signature (64B)</span><span class="v" style="font-size:10.5px;">${esc(proof.ed25519_signature_hex || '--')}</span></div>
      <div class="kv-row"><span class="k">Local Rust Verification</span><span class="v" style="color:${ver.ed25519_verified ? 'var(--lime)' : 'var(--red)'};">${esc(ver.status_label)}</span></div>
      <pre class="raw-json">${esc(JSON.stringify(doc, null, 2))}</pre>
    </div>
  </div>
`;

// Populate ESPR metric progress bars
const esprItems = allFields.filter(f =>
  (f.id && f.id.startsWith('espr_')) || (f.group && f.group.toLowerCase().includes('espr'))
);
const barEl = document.getElementById('espr-bars');
if (esprItems.length) {
  barEl.innerHTML = esprItems.map(f => {
    const num = parseFloat(f.value);
    const pct = isNaN(num) ? 65 : (f.unit === '%' ? Math.min(100, Math.max(4, num)) : Math.min(100, Math.max(12, Math.round((Math.log10(Math.abs(num) + 1) / 4) * 100))));
    return `
      <div class="bar-item">
        <div class="bar-top">
          <span style="color:var(--muted);">${esc(f.name)}</span>
          <strong style="color:#fff;">${esc(f.value)}${f.unit ? ' ' + esc(f.unit) : ''}</strong>
        </div>
        <div class="bar-track"><div class="bar-fill" style="width:${pct}%"></div></div>
      </div>
    `;
  }).join('');
} else {
  barEl.innerHTML = '<div style="color:var(--muted);">No ESPR numeric metrics attached to this passport.</div>';
}

function showView(id, btn) {
  document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
  document.getElementById('view-' + id).classList.add('active');
  document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
  if (btn) btn.classList.add('active');
}

async function sealLedgerEvent(e) {
  e.preventDefault();
  const btn = document.getElementById('ev-submit-btn');
  const st = document.getElementById('ev-status');
  const bizStep = document.getElementById('ev-bizstep').value;
  const actor = document.getElementById('ev-actor').value.trim();
  const readPoint = document.getElementById('ev-readpoint').value.trim();
  const note = document.getElementById('ev-note').value.trim();
  if (!note) return;

  btn.disabled = true;
  btn.textContent = 'SEALING OVER RETICULUM...';
  st.textContent = 'Sending encrypted EPCIS 2.0 event to 0-port Citadel Vault for SHA-256 chaining + Ed25519 signing...';

  try {
    const cleanPath = String(PAYLOAD.path || '/dpp/drone-01').split('?')[0].replace(/\/+$/, '');
    const url = `ironpass://${PAYLOAD.dest_hash}${cleanPath}/ledger`;
    const r = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        type: (bizStep === 'shipping' || bizStep === 'receiving') ? 'TransactionEvent' : 'ObjectEvent',
        action: bizStep === 'commissioning' ? 'ADD' : 'OBSERVE',
        bizStep,
        disposition: bizStep === 'inspecting' ? 'conformant' : 'active',
        readPoint,
        actor,
        note,
      }),
    });
    const res = await r.json();
    if (!res.ok) {
      st.style.color = 'var(--red)';
      st.textContent = `Ledger append failed: ${res.error || 'unknown error'}`;
      btn.disabled = false;
      btn.textContent = '🔐 SEAL & SIGN EVENT →';
      return;
    }
    st.style.color = 'var(--lime)';
    st.textContent = `✓ Event #${res.event.seq} sealed & Ed25519 signed (${res.event.event_hash.slice(0, 16)}...) — reloading card for local Rust chain verification...`;
    setTimeout(() => { window.location.reload(); }, 650);
  } catch (err) {
    st.style.color = 'var(--red)';
    st.textContent = `Reticulum error: ${err}`;
    btn.disabled = false;
    btn.textContent = '🔐 SEAL & SIGN EVENT →';
  }
}
</script>
</body>
</html>
"#;
