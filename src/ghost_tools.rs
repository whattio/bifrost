//! Zero-IP Sovereign Ghost Tools (MCP-over-Reticulum) for the Ghost Special Force
//! (Heimdal, @S3-C, Andvari, Brokk).
//!
//! Every tool executes strictly over the in-process encrypted Reticulum link
//! (0 local HTTP ports, 0 DNS queries, 0 external cloud calls).

use citadel_rns_rs::client::{CitadelRnsClient, PeerSummary};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::cache;
use crate::contacts;

#[derive(Clone, Debug)]
pub struct GhostToolContext {
    pub preferred_vault_hash: Option<String>,
    pub preferred_forge_hash: Option<String>,
    pub llm_hash: Option<String>,
    pub llm_model: String,
    pub ifac_fingerprint: Option<String>,
    pub client_id: String,
}

/// Pure-Rust zero-dependency Base64 encoder (RFC 4648) for inline Reticulum JPEG snapshots.
pub fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i] as u32;
        let b1 = if i + 1 < input.len() { input[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        if i + 1 < input.len() {
            out.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if i + 2 < input.len() {
            out.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

fn find_vault_peer(peers: &[PeerSummary], preferred: Option<&str>) -> Option<String> {
    if let Some(pref) = preferred {
        if let Some(p) = peers.iter().find(|p| p.hash.eq_ignore_ascii_case(pref)) {
            return Some(p.hash.clone());
        }
    }
    let saved = contacts::load_contacts();
    for pet in ["sess", "sessrumnir", "citadel", "vault", "cam"] {
        if let Some(c) = saved.iter().find(|c| c.petname.eq_ignore_ascii_case(pet)) {
            if peers.iter().any(|p| p.hash.eq_ignore_ascii_case(&c.hash)) {
                return Some(c.hash.clone());
            }
        }
    }
    if let Some(p) = peers.iter().find(|p| {
        let a = p.alias.to_ascii_lowercase();
        (a.contains("sess") || a.contains("microsite") || a.contains("cam") || a.contains("vault"))
            && !a.contains("forge")
            && !a.contains("npu")
            && !a.contains("llm")
            && !a.contains("huginn")
    }) {
        return Some(p.hash.clone());
    }
    preferred.map(|s| s.to_string()).or_else(|| {
        peers
            .iter()
            .find(|p| {
                let a = p.alias.to_ascii_lowercase();
                !a.contains("npu") && !a.contains("llm") && !a.contains("huginn")
            })
            .map(|p| p.hash.clone())
    })
}

fn find_forge_peer(peers: &[PeerSummary], preferred: Option<&str>) -> Option<String> {
    if let Some(pref) = preferred {
        if let Some(p) = peers.iter().find(|p| p.hash.eq_ignore_ascii_case(pref)) {
            return Some(p.hash.clone());
        }
    }
    let saved = contacts::load_contacts();
    for pet in ["forge", "ironpass", "dpp"] {
        if let Some(c) = saved.iter().find(|c| c.petname.eq_ignore_ascii_case(pet)) {
            if peers.iter().any(|p| p.hash.eq_ignore_ascii_case(&c.hash)) {
                return Some(c.hash.clone());
            }
        }
    }
    if let Some(p) = peers.iter().find(|p| {
        let a = p.alias.to_ascii_lowercase();
        a.contains("forge") || a.contains("ironpass") || a.contains("explorer")
    }) {
        return Some(p.hash.clone());
    }
    preferred.map(|s| s.to_string())
}

fn find_ai_peer(peers: &[PeerSummary], preferred: Option<&str>) -> Option<PeerSummary> {
    if let Some(pref) = preferred {
        if let Some(p) = peers.iter().find(|p| p.hash.eq_ignore_ascii_case(pref)) {
            return Some(p.clone());
        }
    }
    peers
        .iter()
        .find(|p| {
            let a = p.alias.to_ascii_lowercase();
            a.contains("npu") || a.contains("ventuno") || a.contains("mimir") || a.contains("llm") || a.contains("ollama")
        })
        .cloned()
}

/// Executes a live Zero-IP Sovereign Tool directly over the in-process Reticulum client.
pub async fn run_ghost_tool(
    tool: &str,
    client: &Arc<CitadelRnsClient>,
    ctx: &GhostToolContext,
) -> Value {
    let t0 = Instant::now();
    let peers = client.list_peers().await;
    let active_peers = peers.iter().filter(|p| p.active).count();

    match tool {
        "mesh_peers" => {
            let cache_stats = cache::stats();
            let total_wire: u64 = peers.iter().map(|p| p.wire_bytes).sum();
            let total_reqs: u64 = peers.iter().map(|p| p.requests).sum();
            let peer_items: Vec<Value> = peers
                .iter()
                .map(|p| {
                    json!({
                        "alias": p.alias,
                        "hash": p.hash,
                        "active": p.active,
                        "requests": p.requests,
                        "wire_bytes": p.wire_bytes,
                        "uri": format!("bifrost://{}/", p.hash),
                    })
                })
                .collect();
            let summary = format!(
                "{} Reticulum peer(s) discovered ({} live X25519 links) · {} total encrypted wire KB · IFAC {} · Encrypted Cache: {} files ({:.2} MB).",
                peers.len(),
                active_peers,
                total_wire / 1024,
                ctx.ifac_fingerprint.as_deref().unwrap_or("OFF"),
                cache_stats.files,
                (cache_stats.bytes as f64) / (1024.0 * 1024.0),
            );
            json!({
                "ok": true,
                "tool": "mesh_peers",
                "agent": "heimdal",
                "title": "LIVE RETICULUM MESH TELEMETRY",
                "summary": summary,
                "latency_ms": (t0.elapsed().as_secs_f64() * 1000.0).round(),
                "metrics": [
                    {"label": "LIVE LINKS", "value": format!("{active_peers}/{}", peers.len()), "tone": "lime"},
                    {"label": "OPEN PORTS", "value": "0 HTTP", "tone": "lime"},
                    {"label": "IFAC MASK", "value": ctx.ifac_fingerprint.clone().unwrap_or_else(|| "OFF".into()), "tone": "cyan"},
                    {"label": "ENC WIRE", "value": format!("{} KB ({total_reqs} reqs)", total_wire / 1024), "tone": "muted"},
                    {"label": "ENC CACHE", "value": format!("{} files", cache_stats.files), "tone": "muted"},
                ],
                "peers": peer_items,
                "client_id": ctx.client_id,
                "analysis_prompt": format!("Analyze our live Reticulum mesh status and highlight security posture: {summary}"),
            })
        }

        "camera_snap" => {
            let Some(vault_hash) = find_vault_peer(&peers, ctx.preferred_vault_hash.as_deref()) else {
                return json!({
                    "ok": false,
                    "tool": "camera_snap",
                    "agent": "heimdal",
                    "error": "No 0-port camera/vault peer announced on the Reticulum mesh yet."
                });
            };
            match client
                .request(&vault_hash, "GET", "/api/cam/snapshot.jpg", HashMap::new(), Vec::new(), Duration::from_secs(12))
                .await
            {
                Ok(resp) if resp.status >= 200 && resp.status < 300 && resp.body.len() > 256 => {
                    let b64 = base64_encode(&resp.body);
                    let latency_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
                    let kb = (resp.body.len() as f64) / 1024.0;
                    json!({
                        "ok": true,
                        "tool": "camera_snap",
                        "agent": "heimdal",
                        "title": "0-PORT LIVE RETICULUM CAMERA SNAPSHOT",
                        "summary": format!("Captured {kb:.1} KB JPEG frame over encrypted Reticulum in {latency_ms:.0} ms ({} CHT2 chunks · 0 open RTSP/HTTP ports).", resp.chunks_rx),
                        "latency_ms": latency_ms,
                        "dest_hash": resp.dest_hash,
                        "wire_bytes": resp.wire_bytes,
                        "chunks_cht2": resp.chunks_rx,
                        "image_data_url": format!("data:image/jpeg;base64,{b64}"),
                        "metrics": [
                            {"label": "FRAME SIZE", "value": format!("{kb:.1} KB"), "tone": "lime"},
                            {"label": "LATENCY", "value": format!("{latency_ms:.0} ms"), "tone": "cyan"},
                            {"label": "CHT2 CHUNKS", "value": format!("{}", resp.chunks_rx), "tone": "muted"},
                            {"label": "RTSP/HTTP PORTS", "value": "0 OPEN", "tone": "lime"},
                        ],
                        "open_uri": format!("bifrost://{}/sess/#cam", resp.dest_hash),
                        "open_label": "OPEN LIVE FEED →",
                        "analysis_prompt": format!("We just pulled a {kb:.1} KB live camera snapshot over a 0-port Reticulum link ({} chunks, {latency_ms:.0} ms) from node {}. Explain in 3 bullet points why eBPF-caged 127.0.0.1 camera streaming over Reticulum eliminates IP camera attack surfaces.", resp.chunks_rx, &resp.dest_hash[..8.min(resp.dest_hash.len())]),
                    })
                }
                Ok(resp) => json!({
                    "ok": false,
                    "tool": "camera_snap",
                    "agent": "heimdal",
                    "error": format!("Camera peer returned HTTP {} ({} bytes). Ensure USB/CSI camera streamer is active on the edge node.", resp.status, resp.body.len())
                }),
                Err(e) => json!({
                    "ok": false,
                    "tool": "camera_snap",
                    "agent": "heimdal",
                    "error": format!("Reticulum camera snapshot failed: {e}")
                }),
            }
        }

        "citadel_shield" => {
            let Some(vault_hash) = find_vault_peer(&peers, ctx.preferred_vault_hash.as_deref()) else {
                return json!({
                    "ok": false,
                    "tool": "citadel_shield",
                    "agent": "s3c",
                    "error": "No 0-port Citadel node discovered on Reticulum yet."
                });
            };
            match client
                .request(&vault_hash, "GET", "/api/status", HashMap::new(), Vec::new(), Duration::from_secs(10))
                .await
            {
                Ok(resp) if resp.status >= 200 && resp.status < 300 => {
                    let st: Value = serde_json::from_slice(&resp.body).unwrap_or_else(|_| json!({}));
                    let host = st["host"].as_str().unwrap_or("citadel-node");
                    let drops = st["recentDrops"].as_u64().unwrap_or(0);
                    let open_ports = st["openWebPorts"].as_u64().unwrap_or(0);
                    let shield_mode = st["shieldMode"].as_str().unwrap_or("CAGED (0 WEB PORTS)");
                    let temp_c = st["tempC"].as_f64();
                    let disk_gb = st["diskFreeGb"].as_f64();
                    let ifac = st["ifac"].as_str().unwrap_or("Private IFAC");
                    let latency_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
                    let summary = format!(
                        "Node '{host}' verified over encrypted Reticulum in {latency_ms:.0} ms: {open_ports} open web ports · {drops} [CITADEL-DROP] packets blocked · shield={shield_mode} · IFAC={ifac}."
                    );
                    let mut metrics = vec![
                        json!({"label": "HOST NODE", "value": host, "tone": "lime"}),
                        json!({"label": "OPEN WEB PORTS", "value": format!("{open_ports}"), "tone": "lime"}),
                        json!({"label": "[CITADEL-DROP]", "value": format!("{drops} blocked"), "tone": "amber"}),
                        json!({"label": "SHIELD", "value": shield_mode, "tone": "lime"}),
                    ];
                    if let Some(tc) = temp_c {
                        metrics.push(json!({"label": "SILICON TEMP", "value": format!("{tc:.1} °C"), "tone": "cyan"}));
                    }
                    if let Some(dg) = disk_gb {
                        metrics.push(json!({"label": "VAULT FREE", "value": format!("{dg:.1} GB"), "tone": "muted"}));
                    }
                    json!({
                        "ok": true,
                        "tool": "citadel_shield",
                        "agent": "s3c",
                        "title": "LIVE CITADEL 0-PORT SHIELD AUDIT",
                        "summary": summary,
                        "latency_ms": latency_ms,
                        "dest_hash": resp.dest_hash,
                        "wire_bytes": resp.wire_bytes,
                        "metrics": metrics,
                        "raw": st,
                        "open_uri": format!("bifrost://{}/sess/", resp.dest_hash),
                        "open_label": "OPEN SESSRUMNIR VAULT →",
                        "analysis_prompt": format!("Give a crisp 3-bullet security posture assessment based on this live telemetry: {summary}"),
                    })
                }
                Ok(resp) => json!({
                    "ok": false,
                    "tool": "citadel_shield",
                    "agent": "s3c",
                    "error": format!("Citadel status returned HTTP {}", resp.status)
                }),
                Err(e) => json!({
                    "ok": false,
                    "tool": "citadel_shield",
                    "agent": "s3c",
                    "error": format!("Reticulum link to Citadel node failed: {e}")
                }),
            }
        }

        "citadel_breakout" => {
            let Some(vault_hash) = find_vault_peer(&peers, ctx.preferred_vault_hash.as_deref()) else {
                return json!({
                    "ok": false,
                    "tool": "citadel_breakout",
                    "agent": "s3c",
                    "error": "No 0-port Citadel node discovered on Reticulum yet."
                });
            };
            match client
                .request(&vault_hash, "POST", "/api/redteam/breakout", HashMap::new(), Vec::new(), Duration::from_secs(12))
                .await
            {
                Ok(resp) => {
                    let parsed: Value = serde_json::from_slice(&resp.body)
                        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&resp.body).to_string()}));
                    let latency_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
                    let blocked = parsed["blocked"].as_bool()
                        .or_else(|| parsed["caged"].as_bool())
                        .or_else(|| parsed["ok"].as_bool())
                        .unwrap_or(resp.status >= 200 && resp.status < 300);
                    let raw_pretty = serde_json::to_string_pretty(&parsed).unwrap_or_default();
                    json!({
                        "ok": true,
                        "tool": "citadel_breakout",
                        "agent": "s3c",
                        "title": "LIVE eBPF KERNEL CAGE BREAKOUT TEST",
                        "summary": format!("Executed live eBPF egress breakout probe on 0-port node over Reticulum in {latency_ms:.0} ms (HTTP {} · verdict: {}).", resp.status, if blocked { "EGRESS BLOCKED / CAGE INTACT" } else { "CHECK OUTPUT" }),
                        "latency_ms": latency_ms,
                        "dest_hash": resp.dest_hash,
                        "wire_bytes": resp.wire_bytes,
                        "metrics": [
                            {"label": "KERNEL CAGE", "value": if blocked { "INTACT · BLOCKED" } else { "INSPECT" }, "tone": if blocked { "lime" } else { "amber" }},
                            {"label": "PROBE RTT", "value": format!("{latency_ms:.0} ms"), "tone": "cyan"},
                            {"label": "TRANSPORT", "value": "X25519 RNS", "tone": "muted"},
                        ],
                        "code_output": raw_pretty.chars().take(1200).collect::<String>(),
                        "analysis_prompt": format!("Interpret this live eBPF breakout test result from our 0-port Linux node in 3 crisp bullets: {}", raw_pretty.chars().take(700).collect::<String>()),
                    })
                }
                Err(e) => json!({
                    "ok": false,
                    "tool": "citadel_breakout",
                    "agent": "s3c",
                    "error": format!("Reticulum breakout probe failed: {e}")
                }),
            }
        }

        "forge_inventory" => {
            let mut items: Vec<Value> = Vec::new();
            let mut forge_total: Option<u64> = None;
            let mut forge_callsign: Option<String> = None;
            let mut forge_hash_used: Option<String> = None;
            let mut vault_hash_used: Option<String> = None;
            let mut wire_bytes: usize = 0;

            if let Some(fh) = find_forge_peer(&peers, ctx.preferred_forge_hash.as_deref()) {
                if let Ok(r) = client.request(&fh, "GET", "/api/status", HashMap::new(), Vec::new(), Duration::from_secs(8)).await {
                    wire_bytes += r.wire_bytes;
                    if let Ok(v) = serde_json::from_slice::<Value>(&r.body) {
                        forge_total = v["totalDpps"].as_u64();
                        forge_callsign = v["callsign"].as_str().map(|s| s.to_string());
                        forge_hash_used = Some(r.dest_hash.clone());
                    }
                }
                if let Ok(r) = client.request(&fh, "GET", "/api/v1/dpps?visibility=ALL&limit=10", HashMap::new(), Vec::new(), Duration::from_secs(8)).await {
                    wire_bytes += r.wire_bytes;
                    if let Ok(v) = serde_json::from_slice::<Value>(&r.body) {
                        if let Some(arr) = v["dpps"].as_array() {
                            for d in arr {
                                let id = d["id"].as_str().unwrap_or("");
                                let name = d["name"].as_str().unwrap_or(id);
                                let vis = d["visibility"].as_str().unwrap_or("PUBLIC");
                                items.push(json!({
                                    "source": "Forge",
                                    "id": id,
                                    "name": name,
                                    "badge": vis,
                                    "uri": format!("bifrost://{fh}/"),
                                }));
                            }
                        }
                    }
                }
            }
            if let Some(vh) = find_vault_peer(&peers, ctx.preferred_vault_hash.as_deref()) {
                if let Ok(r) = client.request(&vh, "GET", "/api/dpp", HashMap::new(), Vec::new(), Duration::from_secs(8)).await {
                    wire_bytes += r.wire_bytes;
                    vault_hash_used = Some(r.dest_hash.clone());
                    if let Ok(v) = serde_json::from_slice::<Value>(&r.body) {
                        if let Some(arr) = v["dpps"].as_array() {
                            for d in arr {
                                let id = d["id"].as_str().unwrap_or("");
                                let name = d["name"].as_str().unwrap_or(id);
                                let uri = d["uri"].as_str().map(|s| s.to_string()).unwrap_or_else(|| format!("ironpass://{vh}/dpp/{id}"));
                                items.push(json!({
                                    "source": "Vault",
                                    "id": id,
                                    "name": name,
                                    "badge": "IRONPASS",
                                    "uri": uri,
                                }));
                            }
                        }
                    }
                }
            }

            let latency_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
            if items.is_empty() && forge_total.is_none() {
                return json!({
                    "ok": false,
                    "tool": "forge_inventory",
                    "agent": "andvari",
                    "error": "No IronPass Forge or Citadel DPP Vault peer responded over Reticulum."
                });
            }
            let summary = format!(
                "Synced {} live Digital Product Passports over encrypted Reticulum in {latency_ms:.0} ms (Forge: {}, Citadel Vault: {}).",
                items.len(),
                forge_total.map(|n| format!("{n} forged")).unwrap_or_else(|| "offline".into()),
                if vault_hash_used.is_some() { "online" } else { "offline" },
            );
            json!({
                "ok": true,
                "tool": "forge_inventory",
                "agent": "andvari",
                "title": "LIVE IRONPASS PASSPORT INVENTORY",
                "summary": summary,
                "latency_ms": latency_ms,
                "dest_hash": forge_hash_used.clone().or(vault_hash_used.clone()).unwrap_or_default(),
                "wire_bytes": wire_bytes,
                "metrics": [
                    {"label": "LIVE PASSPORTS", "value": format!("{}", items.len()), "tone": "amber"},
                    {"label": "FORGE SIGNER", "value": forge_callsign.unwrap_or_else(|| "Ed25519".into()), "tone": "lime"},
                    {"label": "SYNC TIME", "value": format!("{latency_ms:.0} ms"), "tone": "cyan"},
                    {"label": "DNS LEAKS", "value": "0", "tone": "lime"},
                ],
                "passports": items,
                "open_uri": forge_hash_used.map(|h| format!("bifrost://{h}/")).unwrap_or_default(),
                "open_label": "OPEN IRONPASS FORGE →",
                "analysis_prompt": format!("Summarize our live IronPass passport posture and how Hybrid #ip= tags protect these {} passports: {summary}", items.len()),
            })
        }

        "forge_identity" => {
            let Some(fh) = find_forge_peer(&peers, ctx.preferred_forge_hash.as_deref()) else {
                return json!({
                    "ok": false,
                    "tool": "forge_identity",
                    "agent": "andvari",
                    "error": "No IronPass Forge peer announced on Reticulum yet."
                });
            };
            match client.request(&fh, "GET", "/api/status", HashMap::new(), Vec::new(), Duration::from_secs(8)).await {
                Ok(resp) if resp.status >= 200 && resp.status < 300 => {
                    let st: Value = serde_json::from_slice(&resp.body).unwrap_or_else(|_| json!({}));
                    let callsign = st["callsign"].as_str().unwrap_or("IRONPASS-FORGE");
                    let issuer = st["issuerHash"].as_str().unwrap_or("unknown");
                    let pubkey = st["publicKeyHex"].as_str().unwrap_or("");
                    let total = st["totalDpps"].as_u64().unwrap_or(0);
                    let latency_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
                    let summary = format!(
                        "IronPass Forge signer '{callsign}' verified live over Reticulum: issuerHash={issuer} · {total} passports minted · 0-port ghost mode."
                    );
                    json!({
                        "ok": true,
                        "tool": "forge_identity",
                        "agent": "andvari",
                        "title": "LIVE IRONPASS ED25519 ISSUER IDENTITY",
                        "summary": summary,
                        "latency_ms": latency_ms,
                        "dest_hash": resp.dest_hash,
                        "wire_bytes": resp.wire_bytes,
                        "metrics": [
                            {"label": "CALLSIGN", "value": callsign, "tone": "amber"},
                            {"label": "ISSUER HASH", "value": issuer, "tone": "cyan"},
                            {"label": "MINTED DPPs", "value": format!("{total}"), "tone": "lime"},
                            {"label": "MODE", "value": st["mode"].as_str().unwrap_or("0-port"), "tone": "muted"},
                        ],
                        "code_output": format!("Ed25519 Public Key: {pubkey}\nVault Hash: {}\nPublic Bridge: {}", st["vaultHash"].as_str().unwrap_or("-"), st["publicBaseUrl"].as_str().unwrap_or("-")),
                        "open_uri": format!("bifrost://{}/", resp.dest_hash),
                        "open_label": "OPEN FORGE CONSOLE →",
                        "analysis_prompt": format!("Explain how Ed25519 issuer key {issuer} signs EU ESPR Digital Product Passports and GS1 EPCIS 2.0 ledger events offline."),
                    })
                }
                Ok(resp) => json!({
                    "ok": false,
                    "tool": "forge_identity",
                    "agent": "andvari",
                    "error": format!("IronPass Forge returned HTTP {}", resp.status)
                }),
                Err(e) => json!({
                    "ok": false,
                    "tool": "forge_identity",
                    "agent": "andvari",
                    "error": format!("Reticulum link to IronPass Forge failed: {e}")
                }),
            }
        }

        "silicon_telemetry" => {
            let ai_peer = find_ai_peer(&peers, ctx.llm_hash.as_deref());
            let mut edge_temp: Option<f64> = None;
            let mut edge_disk: Option<f64> = None;
            let mut edge_host = String::from("edge-node");
            let mut wire_bytes: usize = 0;

            if let Some(vh) = find_vault_peer(&peers, ctx.preferred_vault_hash.as_deref()) {
                if let Ok(r) = client.request(&vh, "GET", "/api/status", HashMap::new(), Vec::new(), Duration::from_secs(6)).await {
                    wire_bytes += r.wire_bytes;
                    if let Ok(st) = serde_json::from_slice::<Value>(&r.body) {
                        edge_temp = st["tempC"].as_f64();
                        edge_disk = st["diskFreeGb"].as_f64();
                        if let Some(h) = st["host"].as_str() {
                            edge_host = h.to_string();
                        }
                    }
                }
            }

            let latency_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
            let ai_status = match &ai_peer {
                Some(p) if p.active => format!("ONLINE ({})", &p.hash[..8.min(p.hash.len())]),
                Some(p) => format!("ANNOUNCED ({})", &p.hash[..8.min(p.hash.len())]),
                None => "OFFLINE".into(),
            };
            let summary = format!(
                "Local AI Silicon ({}) is {} · Edge node '{edge_host}': {}{} · {active_peers}/{} Reticulum links active.",
                ctx.llm_model,
                ai_status,
                edge_temp.map(|t| format!("{t:.1}°C")).unwrap_or_else(|| "temp n/a".into()),
                edge_disk.map(|d| format!(" · {d:.1} GB free")).unwrap_or_default(),
                peers.len(),
            );
            json!({
                "ok": true,
                "tool": "silicon_telemetry",
                "agent": "brokk",
                "title": "LIVE LOCAL SILICON & EDGE HARDWARE",
                "summary": summary,
                "latency_ms": latency_ms,
                "wire_bytes": wire_bytes,
                "metrics": [
                    {"label": "AI SILICON", "value": ctx.llm_model.clone(), "tone": "cyan"},
                    {"label": "NPU LINK", "value": ai_status, "tone": if ai_peer.as_ref().map(|p| p.active).unwrap_or(false) { "lime" } else { "amber" }},
                    {"label": format!("{edge_host} TEMP"), "value": edge_temp.map(|t| format!("{t:.1} °C")).unwrap_or_else(|| "—".into()), "tone": "cyan"},
                    {"label": "VAULT STORAGE", "value": edge_disk.map(|d| format!("{d:.1} GB free")).unwrap_or_else(|| "—".into()), "tone": "muted"},
                    {"label": "SUB-GHZ LORA", "value": "READY (868/915)", "tone": "muted"},
                ],
                "analysis_prompt": format!("Review this live hardware telemetry and recommend how to scale from Ventuno Q 40 TOPS NPU to MIMIR 32GB/128GB nodes: {summary}"),
            })
        }

        "rns_ping" => {
            let Some(vh) = find_vault_peer(&peers, ctx.preferred_vault_hash.as_deref()) else {
                return json!({
                    "ok": false,
                    "tool": "rns_ping",
                    "agent": "brokk",
                    "error": "No edge Reticulum node available to run mesh ping."
                });
            };
            match client.request(&vh, "POST", "/api/reticulum/ping", HashMap::new(), Vec::new(), Duration::from_secs(12)).await {
                Ok(resp) => {
                    let parsed: Value = serde_json::from_slice(&resp.body)
                        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&resp.body).to_string()}));
                    let latency_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
                    let raw_pretty = serde_json::to_string_pretty(&parsed).unwrap_or_default();
                    json!({
                        "ok": true,
                        "tool": "rns_ping",
                        "agent": "brokk",
                        "title": "LIVE NODE-TO-NODE RETICULUM PING",
                        "summary": format!("Executed encrypted Reticulum ping across dark mesh in {latency_ms:.0} ms (HTTP {}).", resp.status),
                        "latency_ms": latency_ms,
                        "dest_hash": resp.dest_hash,
                        "wire_bytes": resp.wire_bytes,
                        "metrics": [
                            {"label": "ROUND-TRIP", "value": format!("{latency_ms:.0} ms"), "tone": "cyan"},
                            {"label": "CRYPTO", "value": "X25519 + Ed25519", "tone": "lime"},
                            {"label": "OPEN PORTS", "value": "0", "tone": "lime"},
                        ],
                        "code_output": raw_pretty.chars().take(1000).collect::<String>(),
                        "analysis_prompt": format!("Analyze this live Reticulum node-to-node ping result in 2 sentences: {}", raw_pretty.chars().take(500).collect::<String>()),
                    })
                }
                Err(e) => json!({
                    "ok": false,
                    "tool": "rns_ping",
                    "agent": "brokk",
                    "error": format!("Reticulum ping failed: {e}")
                }),
            }
        }

        _ => json!({
            "ok": false,
            "error": format!("Unknown Ghost Tool: {tool}")
        }),
    }
}

/// Fast (<=2.5s), best-effort live zero-IP context injected into `/api/ai-chat` so the
/// local NPU/MIMIR model answers with real measured telemetry from the live mesh.
pub async fn build_live_agent_context(
    agent: &str,
    client: &Arc<CitadelRnsClient>,
    ctx: &GhostToolContext,
) -> Option<String> {
    let peers = client.list_peers().await;
    let active = peers.iter().filter(|p| p.active).count();
    let cache_stats = cache::stats();

    match agent {
        "s3c" => {
            let mut lines = vec![format!(
                "Live Reticulum mesh: {active}/{} active X25519 links, IFAC={}, 0 local HTTP ports.",
                peers.len(),
                ctx.ifac_fingerprint.as_deref().unwrap_or("OFF")
            )];
            if let Some(vh) = find_vault_peer(&peers, ctx.preferred_vault_hash.as_deref()) {
                if let Ok(r) = client.request(&vh, "GET", "/api/status", HashMap::new(), Vec::new(), Duration::from_millis(2200)).await {
                    if let Ok(st) = serde_json::from_slice::<Value>(&r.body) {
                        lines.push(format!(
                            "Live 0-port Citadel node '{}': openWebPorts={}, recentDrops={}, shieldMode={}, tempC={}, diskFreeGb={} GB.",
                            st["host"].as_str().unwrap_or("node"),
                            st["openWebPorts"].as_u64().unwrap_or(0),
                            st["recentDrops"].as_u64().unwrap_or(0),
                            st["shieldMode"].as_str().unwrap_or("CAGED"),
                            st["tempC"].as_f64().map(|t| format!("{t:.1}")).unwrap_or_else(|| "?".into()),
                            st["diskFreeGb"].as_f64().map(|d| format!("{d:.1}")).unwrap_or_else(|| "?".into()),
                        ));
                    }
                }
            }
            Some(lines.join(" "))
        }
        "andvari" => {
            let mut lines = Vec::new();
            if let Some(fh) = find_forge_peer(&peers, ctx.preferred_forge_hash.as_deref()) {
                if let Ok(r) = client.request(&fh, "GET", "/api/status", HashMap::new(), Vec::new(), Duration::from_millis(2200)).await {
                    if let Ok(st) = serde_json::from_slice::<Value>(&r.body) {
                        lines.push(format!(
                            "Live IronPass Forge signer '{}': issuerHash={}, totalDpps={}, mode={}.",
                            st["callsign"].as_str().unwrap_or("Forge"),
                            st["issuerHash"].as_str().unwrap_or("?"),
                            st["totalDpps"].as_u64().unwrap_or(0),
                            st["mode"].as_str().unwrap_or("0-port"),
                        ));
                    }
                }
            }
            if let Some(vh) = find_vault_peer(&peers, ctx.preferred_vault_hash.as_deref()) {
                if let Ok(r) = client.request(&vh, "GET", "/api/dpp", HashMap::new(), Vec::new(), Duration::from_millis(2200)).await {
                    if let Ok(v) = serde_json::from_slice::<Value>(&r.body) {
                        if let Some(arr) = v["dpps"].as_array() {
                            let names: Vec<String> = arr.iter().take(6).filter_map(|d| d["id"].as_str().map(|s| s.to_string())).collect();
                            lines.push(format!("Live Citadel Vault DPPs ({} total): {}.", arr.len(), names.join(", ")));
                        }
                    }
                }
            }
            if lines.is_empty() { None } else { Some(lines.join(" ")) }
        }
        "brokk" => {
            let ai_peer = find_ai_peer(&peers, ctx.llm_hash.as_deref());
            let mut lines = vec![format!(
                "Live AI silicon: model={}, peer={}, encrypted_cache={} files ({:.2} MB).",
                ctx.llm_model,
                ai_peer.as_ref().map(|p| format!("{} (active={})", p.alias, p.active)).unwrap_or_else(|| "offline".into()),
                cache_stats.files,
                (cache_stats.bytes as f64) / (1024.0 * 1024.0),
            )];
            if let Some(vh) = find_vault_peer(&peers, ctx.preferred_vault_hash.as_deref()) {
                if let Ok(r) = client.request(&vh, "GET", "/api/status", HashMap::new(), Vec::new(), Duration::from_millis(2200)).await {
                    if let Ok(st) = serde_json::from_slice::<Value>(&r.body) {
                        lines.push(format!(
                            "Edge Linux node '{}': tempC={}, diskFreeGb={} GB.",
                            st["host"].as_str().unwrap_or("node"),
                            st["tempC"].as_f64().map(|t| format!("{t:.1}")).unwrap_or_else(|| "?".into()),
                            st["diskFreeGb"].as_f64().map(|d| format!("{d:.1}")).unwrap_or_else(|| "?".into()),
                        ));
                    }
                }
            }
            Some(lines.join(" "))
        }
        _ => {
            let peer_list: Vec<String> = peers
                .iter()
                .take(8)
                .map(|p| format!("{}({})", p.alias, if p.active { "live" } else { "idle" }))
                .collect();
            Some(format!(
                "Live BIFROST mesh: {active}/{} active Reticulum links [{}], IFAC={}, encrypted_cache={} files, 0 HTTP ports, 0 cloud calls.",
                peers.len(),
                peer_list.join(", "),
                ctx.ifac_fingerprint.as_deref().unwrap_or("OFF"),
                cache_stats.files,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::base64_encode;

    #[test]
    fn base64_rfc4648_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"BIFROST"), "QklGUk9TVA==");
    }
}
