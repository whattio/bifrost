//! BIFROST — The Reticulum Ghost Browser (`bifrost://`, `ironpass://` & `citadel://`)
//! Copyright (C) 2026 whatt.io — Dual-licensed under AGPL-3.0-or-later OR Commercial License.
//!
//! Security Properties:
//! - ZERO local HTTP ports on the operator workstation (in-process `citadel-rns-rs`)
//! - End-to-end encrypted X25519 + Ed25519 Reticulum Links over `FastTcp`
//! - Hard-blocks all `http://` and `https://` Web2/DNS egress by default
//! - Ephemeral RAM-only Incognito WebView session (zero cookies/history/cache on SSD)
//! - Local Rust verification of Ed25519-signed EU ESPR DPPs & hash-chained GS1 EPCIS 2.0 Ledgers
//! - Optional local Sovereign AI Omnibox backed by any OpenAI-compatible NPU/Ollama/llama.cpp peer

mod cache;
mod contacts;
mod dpp;
mod sandbox;
mod ui;

use anyhow::Result;
use citadel_rns_rs::client::CitadelRnsClient;
use citadel_rns_rs::security::{ClientIdentity, IfacKey};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::{Icon, WindowBuilder};
#[cfg(target_os = "linux")]
use tao::platform::unix::WindowExtUnix;
use tracing::{info, warn};
use wry::http::{header, Request, Response, StatusCode};
use wry::{RequestAsyncResponder, WebContext, WebViewBuilder};
#[cfg(target_os = "linux")]
use wry::WebViewBuilderExtUnix;

#[derive(Parser, Debug, Clone)]
#[command(name = "bifrost-ghost-browser")]
#[command(version = env!("BIFROST_BUILD_TAG"))]
#[command(about = "BIFROST — The Reticulum Ghost Browser (bifrost:// & ironpass://)")]
struct Args {
    /// Optional Zero-IP URI passed as positional argument (e.g. from xdg-mime / OS handler)
    #[arg(value_name = "URI")]
    positional_url: Option<String>,

    /// Reticulum FastTcp listen address for 0-port outbound Ghost Vaults
    #[arg(long, default_value = "127.0.0.1:14290")]
    rns_listen: String,

    /// Disable Reticulum inbound listener (outbound --rns-peer only)
    #[arg(long, default_value_t = false)]
    no_listen: bool,

    /// Optional outbound Reticulum peer(s) to dial directly (repeatable, e.g. 192.168.1.50:14245)
    #[arg(long = "rns-peer")]
    rns_peers: Vec<String>,

    /// Optional pinned 128-bit destination hash allowlist (rejects all other announces)
    #[arg(long = "allow-hash")]
    allow_hashes: Vec<String>,

    /// Deterministic Curve25519 client identity name
    #[arg(long, default_value = "bifrost-ghost-browser")]
    identity_name: String,

    /// Initial Zero-IP URI to open
    #[arg(long, default_value = "bifrost://home/")]
    start_url: String,

    /// Allow standard http:// and https:// Web2 navigation (OFF by default for Ghost isolation)
    #[arg(long, default_value_t = false)]
    allow_web2: bool,

    /// Optional 128-bit Reticulum destination hash of your local Sovereign AI LLM/NPU peer
    #[arg(long, env = "BIFROST_LLM_HASH")]
    llm_hash: Option<String>,

    /// Model alias served by your local OpenAI-compatible AI peer (Ollama / llama.cpp / Ventuno Q)
    #[arg(long, env = "BIFROST_LLM_MODEL", default_value = "qwen3-4b-npu")]
    llm_model: String,

    /// Optional Bearer API key for local LLM peer (also auto-loaded from ~/.config/bifrost/llm_api_key)
    #[arg(long, env = "BIFROST_LLM_API_KEY")]
    llm_api_key: Option<String>,

    /// Lock 1 (IFAC): Path to shared dark-network key file (chmod 600)
    #[arg(long = "ifac-key-file", env = "CITADEL_IFAC_KEY_FILE")]
    ifac_key_file: Option<PathBuf>,

    /// Lock 1 (IFAC): Inline shared network passphrase (prefer --ifac-key-file)
    #[arg(long = "ifac-key", env = "CITADEL_IFAC_KEY")]
    ifac_key: Option<String>,

    /// Lock 2 (Ed25519 Mutual Auth): Optional custom path for persistent client Ed25519 keypair
    #[arg(long = "client-key-file", env = "BIFROST_CLIENT_KEY_FILE")]
    client_key_file: Option<PathBuf>,

    /// Print this device's persistent 32-hex Ed25519 Client ID (for vault --allow-client) and exit
    #[arg(long, default_value_t = false)]
    print_client_id: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Bookmark {
    label: String,
    uri: String,
    #[serde(default)]
    icon: String,
    #[serde(default)]
    tone: String,
}

#[derive(Clone)]
struct MeshCtx {
    client: Arc<CitadelRnsClient>,
    listen_label: Option<String>,
    allow_label: Vec<String>,
    llm_hash: Option<String>,
    llm_model: String,
    llm_key: Option<String>,
}

const ICON_BIFROST_PNG: &[u8] = include_bytes!("../assets/bifrost-128.png");
const ICON_CITADEL_PNG: &[u8] = include_bytes!("../assets/citadel-128.png");
const ICON_IRONPASS_PNG: &[u8] = include_bytes!("../assets/ironpass-128.png");
const ICON_BIFROST_RGBA64: &[u8] = include_bytes!("../assets/bifrost-64.rgba");
const JSQR_BUNDLE: &[u8] = include_bytes!("../assets/jsqr.js");

fn resolve_llm_api_key(cli_opt: Option<String>) -> Option<String> {
    if let Some(k) = cli_opt {
        let trimmed = k.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        for rel in [".config/bifrost/llm_api_key", ".config/citadel/llm_api_key"] {
            let p = PathBuf::from(&home).join(rel);
            if let Ok(contents) = std::fs::read_to_string(p) {
                let trimmed = contents.trim().to_string();
                if !trimmed.is_empty() {
                    return Some(trimmed);
                }
            }
        }
    }
    None
}

fn load_user_bookmarks() -> Vec<Bookmark> {
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".config/bifrost/bookmarks.json");
        if let Ok(contents) = std::fs::read_to_string(p) {
            if let Ok(items) = serde_json::from_str::<Vec<Bookmark>>(&contents) {
                return items;
            }
        }
    }
    Vec::new()
}

fn relays_file_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("bifrost").join("peers.json")
}

fn load_saved_relays() -> Vec<String> {
    let p = relays_file_path();
    let Ok(raw) = std::fs::read_to_string(&p) else {
        return Vec::new();
    };
    if let Ok(list) = serde_json::from_str::<Vec<String>>(&raw) {
        return list
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    Vec::new()
}

fn save_relays(relays: &[String]) {
    let p = relays_file_path();
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut dedup: Vec<String> = Vec::new();
    for r in relays {
        let t = r.trim();
        if !t.is_empty() && !dedup.iter().any(|x| x == t) {
            dedup.push(t.to_string());
        }
    }
    if let Ok(json) = serde_json::to_string_pretty(&dedup) {
        if std::fs::write(&p, json).is_ok() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
            }
        }
    }
}

fn split_think_blocks(raw: &str) -> (String, Option<String>) {
    if let Some(start) = raw.find("<think>") {
        if let Some(end_rel) = raw[start + 7..].find("</think>") {
            let end = start + 7 + end_rel;
            let thinking = raw[start + 7..end].trim().to_string();
            let mut reply = String::new();
            reply.push_str(raw[..start].trim());
            let after = raw[end + 8..].trim();
            if !after.is_empty() {
                if !reply.is_empty() {
                    reply.push_str("\n\n");
                }
                reply.push_str(after);
            }
            return (
                reply,
                if thinking.is_empty() {
                    None
                } else {
                    Some(thinking)
                },
            );
        }
    }
    (raw.trim().to_string(), None)
}

fn parse_custom_uri(uri: &str, scheme: &str) -> (String, String) {
    let prefix = format!("{scheme}://");
    let rest = uri.strip_prefix(&prefix).unwrap_or(uri);
    match rest.find('/') {
        Some(idx) => {
            let host = rest[..idx].to_string();
            let path = rest[idx..].to_string();
            (host, if path.is_empty() { "/".to_string() } else { path })
        }
        None => (rest.to_string(), "/".to_string()),
    }
}

fn parse_query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(url_decode(v));
            }
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(val) = u8::from_str_radix(hex, 16) {
                    out.push(val);
                    i += 3;
                    continue;
                }
            }
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn png_response(bytes: &'static [u8]) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .header("Access-Control-Allow-Origin", "*")
        .header("Cache-Control", "public, max-age=86400")
        .body(bytes.to_vec())
        .unwrap()
}

fn html_response(status: StatusCode, body: String) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header("Access-Control-Allow-Origin", "*")
        .body(body.into_bytes())
        .unwrap()
}

fn json_response(status: StatusCode, body: String) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "*")
        .header("Cache-Control", "no-store")
        .body(body.into_bytes())
        .unwrap()
}

fn cors_preflight_response() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "*")
        .body(Vec::new())
        .unwrap()
}

fn extract_32_hex(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.len() < 32 {
        return None;
    }
    for i in 0..=bytes.len() - 32 {
        let slice = &bytes[i..i + 32];
        if slice.iter().all(|b| b.is_ascii_hexdigit()) {
            let prev_hex = i > 0 && bytes[i - 1].is_ascii_hexdigit();
            let next_hex = i + 32 < bytes.len() && bytes[i + 32].is_ascii_hexdigit();
            if !prev_hex && !next_hex {
                return Some(std::str::from_utf8(slice).unwrap_or("").to_ascii_lowercase());
            }
        }
    }
    None
}

fn upsert_contact_from_fields(
    mut petname: String,
    mut label: String,
    mut hash: String,
    mut scheme: String,
    mut cpath: String,
    mut notes: String,
    card: Option<&str>,
) -> Result<Vec<contacts::Contact>, String> {
    if let Some(card_raw) = card {
        let trimmed = card_raw.trim();
        if let Some(rest) = trimmed.strip_prefix("bifrost-card://") {
            let (main_part, query_part) = match rest.split_once('?') {
                Some((m, q)) => (m, Some(q)),
                None => (rest, None),
            };
            let (id_part, path_part) = match main_part.find('/') {
                Some(idx) => (&main_part[..idx], &main_part[idx..]),
                None => (main_part, "/"),
            };
            if let Some((p, h)) = id_part.split_once('@') {
                if petname.trim().is_empty() {
                    petname = p.to_string();
                }
                if hash.trim().is_empty() {
                    hash = h.to_string();
                }
            } else if hash.trim().is_empty() {
                hash = id_part.to_string();
            }
            if !path_part.is_empty() && cpath == "/" {
                cpath = path_part.to_string();
            }
            if let Some(q) = query_part {
                if let Some(l) = parse_query_param(q, "label") {
                    if label.trim().is_empty() {
                        label = l;
                    }
                }
                if let Some(n) = parse_query_param(q, "notes") {
                    if notes.trim().is_empty() {
                        notes = n;
                    }
                }
                if let Some(s) = parse_query_param(q, "scheme") {
                    if !s.trim().is_empty() {
                        scheme = s;
                    }
                }
            }
        } else if hash.trim().is_empty() {
            if let Some(h) = extract_32_hex(trimmed) {
                hash = h;
            }
        }
    }

    // Allow pasting full bifrost://<32-hex>/... or ironpass://<32-hex>/... directly into the hash box
    if !contacts::is_valid_hash(hash.trim()) {
        if let Some(h) = extract_32_hex(&hash) {
            if hash.trim().starts_with("ironpass://") && scheme == "bifrost" {
                scheme = "ironpass".to_string();
            }
            hash = h;
        }
    }

    // Auto-derive petname from label (or hash prefix) if left blank
    if petname.trim().is_empty() {
        let fallback_src = if !label.trim().is_empty() {
            label.trim().to_string()
        } else if hash.len() >= 8 {
            format!("node-{}", &hash[..8])
        } else {
            String::new()
        };
        let slug: String = fallback_src
            .to_ascii_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        let trimmed_slug = slug.trim_matches('-').to_string();
        if !trimmed_slug.is_empty() {
            petname = trimmed_slug;
        }
    }

    if scheme.trim().is_empty() {
        scheme = "bifrost".to_string();
    }
    if cpath.trim().is_empty() {
        cpath = "/".to_string();
    }

    let entry = contacts::Contact {
        petname,
        label,
        hash,
        scheme,
        path: cpath,
        notes,
        updated_at: 0,
    };
    contacts::upsert_contact(entry)
}

fn spawn_mesh_handler(
    scheme: &'static str,
    rt: Arc<tokio::runtime::Runtime>,
    ctx: MeshCtx,
    request: Request<Vec<u8>>,
    responder: RequestAsyncResponder,
) {
    let uri = request.uri().to_string();
    let method = request.method().as_str().to_string();
    let body = request.body().clone();
    let mut hdrs = HashMap::new();
    for (k, v) in request.headers() {
        if let Ok(vs) = v.to_str() {
            hdrs.insert(k.as_str().to_lowercase(), vs.to_string());
        }
    }

    let (host, path) = parse_custom_uri(&uri, scheme);
    let client = Arc::clone(&ctx.client);

    rt.spawn(async move {
        if method.eq_ignore_ascii_case("OPTIONS") {
            responder.respond(cors_preflight_response());
            return;
        }
        if host.eq_ignore_ascii_case("home") {
            if path == "/" || path == "/index.html" {
                let html = ui::HOME_HTML
                    .replace("{{BIFROST_VERSION}}", env!("BIFROST_VERSION"))
                    .replace("{{BIFROST_SHORT_TAG}}", env!("BIFROST_SHORT_TAG"))
                    .replace("{{BIFROST_BUILD_TAG}}", env!("BIFROST_BUILD_TAG"));
                responder.respond(html_response(StatusCode::OK, html));
                return;
            }
            if path == "/showcase" || path == "/showcase/" {
                responder.respond(html_response(StatusCode::OK, sandbox::SHOWCASE_HTML.to_string()));
                return;
            }
            if path == "/icons/bifrost.png" {
                responder.respond(png_response(ICON_BIFROST_PNG));
                return;
            }
            if path == "/icons/citadel.png" {
                responder.respond(png_response(ICON_CITADEL_PNG));
                return;
            }
            if path == "/icons/ironpass.png" {
                responder.respond(png_response(ICON_IRONPASS_PNG));
                return;
            }
            if path == "/js/jsqr.js" {
                let resp = Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Cache-Control", "public, max-age=86400")
                    .body(JSQR_BUNDLE.to_vec())
                    .unwrap();
                responder.respond(resp);
                return;
            }
            if path == "/qr-sheet" || path == "/qr-sheet/" {
                let peers = client.list_peers().await;
                let vault_hash = peers
                    .iter()
                    .find(|p| {
                        let a = p.alias.to_lowercase();
                        a.contains("ironpass") || a.contains("forge")
                    })
                    .or_else(|| peers.first())
                    .map(|p| p.hash.clone())
                    .unwrap_or_else(|| "local-demo".to_string());

                let mut public_base = "https://example.org/dpp".to_string();
                let mut live_items: Vec<dpp::QrTagItem> = Vec::new();
                if vault_hash != "local-demo" {
                    if let Ok(st) = client
                        .request(&vault_hash, "GET", "/api/status", HashMap::new(), Vec::new(), Duration::from_secs(4))
                        .await
                    {
                        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&st.body) {
                            if let Some(u) = v.get("publicBaseUrl").and_then(|x| x.as_str()) {
                                let trimmed = u.trim().trim_end_matches('/');
                                if !trimmed.is_empty() {
                                    public_base = trimmed.to_string();
                                }
                            }
                        }
                    }
                    if let Ok(resp) = client
                        .request(&vault_hash, "GET", "/api/dpp/my-dpps", HashMap::new(), Vec::new(), Duration::from_secs(6))
                        .await
                    {
                        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&resp.body) {
                            if let Some(arr) = v.get("dpps").and_then(|x| x.as_array()) {
                                for entry in arr {
                                    let id = entry.get("id").and_then(|x| x.as_str()).unwrap_or("").trim();
                                    if id.is_empty() {
                                        continue;
                                    }
                                    let title = entry
                                        .get("productName")
                                        .and_then(|x| x.as_str())
                                        .unwrap_or(id)
                                        .to_string();
                                    let sku = entry
                                        .get("gtin")
                                        .and_then(|x| x.as_str())
                                        .filter(|s| !s.is_empty())
                                        .unwrap_or(id)
                                        .to_string();
                                    live_items.push(dpp::QrTagItem {
                                        id: id.to_string(),
                                        title,
                                        sku,
                                        public_url: format!("{public_base}/{id}"),
                                    });
                                }
                            }
                        }
                    }
                }

                let html = if live_items.is_empty() {
                    dpp::render_qr_sheet_html(&vault_hash)
                } else {
                    dpp::render_qr_sheet_html_with_items(&vault_hash, &live_items)
                };
                responder.respond(html_response(StatusCode::OK, html));
                return;
            }
            if path == "/api/peers" || path.starts_with("/api/peers?") {
                let peers = client.list_peers().await;
                let bookmarks = load_user_bookmarks();
                let detected_llm = ctx.llm_hash.clone().or_else(|| {
                    peers
                        .iter()
                        .find(|p| {
                            let a = p.alias.to_lowercase();
                            a.contains("npu") || a.contains("llm") || a.contains("ollama")
                        })
                        .map(|p| p.hash.clone())
                });
                let relays = client.configured_peers().await;
                let cstats = cache::stats();
                let payload = serde_json::json!({
                    "ok": true,
                    "version": env!("BIFROST_VERSION"),
                    "git_sha": env!("BIFROST_GIT_SHA"),
                    "build_tag": env!("BIFROST_BUILD_TAG"),
                    "client_id": client.client_id(),
                    "client_pubkey_hex": client.client_pubkey_hex(),
                    "ifac_fingerprint": client.ifac_fingerprint(),
                    "identity": client.identity_name(),
                    "rns_listen": ctx.listen_label,
                    "relays": relays,
                    "allow_hashes": ctx.allow_label,
                    "llm_hash": detected_llm,
                    "llm_model": ctx.llm_model,
                    "cache_files": cstats.files,
                    "cache_bytes": cstats.bytes,
                    "bookmarks": bookmarks,
                    "peers": peers,
                });
                responder.respond(json_response(StatusCode::OK, payload.to_string()));
                return;
            }
            if path == "/api/cache/clear" || path.starts_with("/api/cache/clear?") {
                let removed = cache::clear();
                let cstats = cache::stats();
                let out = serde_json::json!({
                    "ok": true,
                    "removed": removed,
                    "cache_files": cstats.files,
                    "cache_bytes": cstats.bytes,
                });
                responder.respond(json_response(StatusCode::OK, out.to_string()));
                return;
            }
            if path == "/api/relays/add" || path.starts_with("/api/relays/add?") {
                let addr = path
                    .split_once('?')
                    .and_then(|(_, q)| parse_query_param(q, "addr"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if addr.is_empty() || !addr.contains(':') {
                    let out = serde_json::json!({ "ok": false, "error": "Enter relay as host:port (e.g. 192.168.1.50:14290)" });
                    responder.respond(json_response(StatusCode::BAD_REQUEST, out.to_string()));
                    return;
                }
                client.connect_peer(&addr).await;
                let mut saved = load_saved_relays();
                if !saved.iter().any(|x| x == &addr) {
                    saved.push(addr.clone());
                    save_relays(&saved);
                }
                let relays = client.configured_peers().await;
                responder.respond(json_response(StatusCode::OK, serde_json::json!({ "ok": true, "relays": relays }).to_string()));
                return;
            }
            if path == "/api/relays/remove" || path.starts_with("/api/relays/remove?") {
                let addr = path
                    .split_once('?')
                    .and_then(|(_, q)| parse_query_param(q, "addr"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let mut saved = load_saved_relays();
                saved.retain(|x| x != &addr);
                save_relays(&saved);
                responder.respond(json_response(StatusCode::OK, serde_json::json!({ "ok": true, "relays": saved }).to_string()));
                return;
            }
            if path == "/api/contacts/save" || path.starts_with("/api/contacts/save?") {
                let q = path.split_once('?').map(|(_, q)| q).unwrap_or("");
                let petname = parse_query_param(q, "petname").unwrap_or_default();
                let label = parse_query_param(q, "label").unwrap_or_default();
                let hash = parse_query_param(q, "hash").unwrap_or_default();
                let scheme = parse_query_param(q, "scheme").unwrap_or_else(|| "bifrost".to_string());
                let cpath = parse_query_param(q, "path").unwrap_or_else(|| "/".to_string());
                let notes = parse_query_param(q, "notes").unwrap_or_default();
                let card = parse_query_param(q, "card");
                match upsert_contact_from_fields(petname, label, hash, scheme, cpath, notes, card.as_deref()) {
                    Ok(list) => responder.respond(json_response(StatusCode::OK, serde_json::json!({ "ok": true, "contacts": list }).to_string())),
                    Err(err) => responder.respond(json_response(StatusCode::BAD_REQUEST, serde_json::json!({ "ok": false, "error": err }).to_string())),
                }
                return;
            }
            if path == "/api/contacts/delete" || path.starts_with("/api/contacts/delete?") {
                let petname = path.split_once('?').and_then(|(_, q)| parse_query_param(q, "petname")).unwrap_or_default();
                match contacts::delete_contact(&petname) {
                    Ok(list) => responder.respond(json_response(StatusCode::OK, serde_json::json!({ "ok": true, "contacts": list }).to_string())),
                    Err(err) => responder.respond(json_response(StatusCode::BAD_REQUEST, serde_json::json!({ "ok": false, "error": err }).to_string())),
                }
                return;
            }
            if path == "/api/contacts" || path.starts_with("/api/contacts?") {
                if method.eq_ignore_ascii_case("GET") {
                    let list = contacts::load_contacts();
                    let payload = serde_json::json!({
                        "ok": true,
                        "file": contacts::contacts_file_path().display().to_string(),
                        "contacts": list,
                    });
                    responder.respond(json_response(StatusCode::OK, payload.to_string()));
                    return;
                }
                if method.eq_ignore_ascii_case("POST") {
                    let req_json: serde_json::Value = match serde_json::from_slice(&body) {
                        Ok(v) => v,
                        Err(e) => {
                            let out = serde_json::json!({ "ok": false, "error": format!("Invalid JSON: {e}") });
                            responder.respond(json_response(StatusCode::BAD_REQUEST, out.to_string()));
                            return;
                        }
                    };
                    let petname = req_json.get("petname").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let label = req_json.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let hash = req_json.get("hash").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let scheme = req_json.get("scheme").and_then(|v| v.as_str()).unwrap_or("bifrost").to_string();
                    let cpath = req_json.get("path").and_then(|v| v.as_str()).unwrap_or("/").to_string();
                    let notes = req_json.get("notes").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let card = req_json.get("card").and_then(|v| v.as_str());
                    match upsert_contact_from_fields(petname, label, hash, scheme, cpath, notes, card) {
                        Ok(list) => responder.respond(json_response(StatusCode::OK, serde_json::json!({ "ok": true, "contacts": list }).to_string())),
                        Err(err) => responder.respond(json_response(StatusCode::BAD_REQUEST, serde_json::json!({ "ok": false, "error": err }).to_string())),
                    }
                    return;
                }
                if method.eq_ignore_ascii_case("DELETE") {
                    let petname = path.split_once('?').and_then(|(_, q)| parse_query_param(q, "petname")).unwrap_or_default();
                    match contacts::delete_contact(&petname) {
                        Ok(list) => responder.respond(json_response(StatusCode::OK, serde_json::json!({ "ok": true, "contacts": list }).to_string())),
                        Err(err) => responder.respond(json_response(StatusCode::BAD_REQUEST, serde_json::json!({ "ok": false, "error": err }).to_string())),
                    }
                    return;
                }
            }
            if (path == "/api/ai-chat" || path.starts_with("/api/ai-chat?"))
                && (method.eq_ignore_ascii_case("POST") || method.eq_ignore_ascii_case("GET"))
            {
                let t0 = Instant::now();
                let query_payload = path.split_once('?').and_then(|(_, q)| parse_query_param(q, "payload"));
                let req_json: serde_json::Value = if !body.is_empty() {
                    match serde_json::from_slice(&body) {
                        Ok(v) => v,
                        Err(e) => {
                            let out = serde_json::json!({ "ok": false, "error": format!("Invalid JSON body: {e}") });
                            responder.respond(json_response(StatusCode::BAD_REQUEST, out.to_string()));
                            return;
                        }
                    }
                } else if let Some(ref qp) = query_payload {
                    match serde_json::from_str(qp) {
                        Ok(v) => v,
                        Err(e) => {
                            let out = serde_json::json!({ "ok": false, "error": format!("Invalid JSON payload param: {e}") });
                            responder.respond(json_response(StatusCode::BAD_REQUEST, out.to_string()));
                            return;
                        }
                    }
                } else {
                    let out = serde_json::json!({ "ok": false, "error": "Empty AI chat payload" });
                    responder.respond(json_response(StatusCode::BAD_REQUEST, out.to_string()));
                    return;
                };

                let model = req_json
                    .get("model")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(&ctx.llm_model)
                    .to_string();
                let enable_thinking = req_json
                    .get("enable_thinking")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let mut messages: Vec<serde_json::Value> = req_json
                    .get("messages")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                let has_system = messages
                    .first()
                    .and_then(|m| m.get("role"))
                    .and_then(|r| r.as_str())
                    .map(|r| r == "system")
                    .unwrap_or(false);

                if !has_system {
                    messages.insert(
                        0,
                        serde_json::json!({
                            "role": "system",
                            "content": "You are a Sovereign AI running locally on the user's own hardware inside an encrypted, zero-IP Bifrost Reticulum network (zero public cloud calls, zero telemetry, zero cookies). Be accurate, direct, and concise."
                        }),
                    );
                }

                let target_hash_opt = {
                    let peers = client.list_peers().await;
                    if let Some(ref pinned) = ctx.llm_hash {
                        Some(pinned.clone())
                    } else {
                        peers
                            .iter()
                            .find(|p| {
                                let a = p.alias.to_lowercase();
                                a.contains("npu") || a.contains("llm") || a.contains("ollama")
                            })
                            .map(|p| p.hash.clone())
                    }
                };

                let Some(target_hash) = target_hash_opt else {
                    let out = serde_json::json!({
                        "ok": false,
                        "error": "No local Sovereign AI peer connected. Start a citadel-vault-daemon pointing at your local Ollama/llama.cpp server and pass --llm-hash <128-bit-hash> (see docs/HOSTING_A_GHOST_SERVER.md)."
                    });
                    responder.respond(json_response(StatusCode::SERVICE_UNAVAILABLE, out.to_string()));
                    return;
                };

                let upstream_payload = serde_json::json!({
                    "model": model,
                    "messages": messages,
                    "max_tokens": if enable_thinking { 1024 } else { 512 },
                    "temperature": 0.6,
                    "chat_template_kwargs": { "enable_thinking": enable_thinking }
                });

                let mut rns_hdrs = HashMap::new();
                rns_hdrs.insert("content-type".into(), "application/json".into());
                rns_hdrs.insert("accept".into(), "application/json".into());
                if let Some(ref key) = ctx.llm_key {
                    rns_hdrs.insert("authorization".into(), format!("Bearer {key}"));
                }

                match client
                    .request(
                        &target_hash,
                        "POST",
                        "/v1/chat/completions",
                        rns_hdrs,
                        upstream_payload.to_string().into_bytes(),
                        Duration::from_secs(90),
                    )
                    .await
                {
                    Ok(resp) => {
                        let latency_ms = (t0.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0;
                        if resp.status < 200 || resp.status >= 300 {
                            let err_txt = String::from_utf8_lossy(&resp.body).to_string();
                            let out = serde_json::json!({
                                "ok": false,
                                "status": resp.status,
                                "error": format!("Local AI peer returned HTTP {}: {}", resp.status, err_txt),
                            });
                            responder.respond(json_response(StatusCode::BAD_GATEWAY, out.to_string()));
                            return;
                        }
                        let parsed: serde_json::Value = match serde_json::from_slice(&resp.body) {
                            Ok(v) => v,
                            Err(e) => {
                                let out = serde_json::json!({
                                    "ok": false,
                                    "error": format!("Invalid JSON from AI peer: {e}"),
                                });
                                responder.respond(json_response(StatusCode::BAD_GATEWAY, out.to_string()));
                                return;
                            }
                        };

                        let msg_obj = &parsed["choices"][0]["message"];
                        let raw_content = msg_obj["content"].as_str().unwrap_or("");
                        let reasoning_field = msg_obj["reasoning_content"]
                            .as_str()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty());
                        let (reply, inline_think) = split_think_blocks(raw_content);
                        let thinking = reasoning_field.or(inline_think);
                        let resp_model = parsed["model"].as_str().unwrap_or(&model);
                        let tok_per_sec = parsed["timings"]["predicted_per_second"]
                            .as_f64()
                            .map(|v| (v * 10.0).round() / 10.0);

                        let out = serde_json::json!({
                            "ok": true,
                            "reply": reply,
                            "thinking": thinking,
                            "model": resp_model,
                            "dest_hash": resp.dest_hash,
                            "wire_bytes": resp.wire_bytes,
                            "chunks_cht2": resp.chunks_rx,
                            "latency_ms": latency_ms,
                            "tok_per_sec": tok_per_sec,
                        });
                        responder.respond(json_response(StatusCode::OK, out.to_string()));
                    }
                    Err(e) => {
                        let out = serde_json::json!({
                            "ok": false,
                            "error": format!("Reticulum link to AI peer ({target_hash}) failed: {e}"),
                        });
                        responder.respond(json_response(StatusCode::BAD_GATEWAY, out.to_string()));
                    }
                }
                return;
            }
            if let Some((_, query)) = path.split_once('?') {
                if path.starts_with("/api/dpp-fetch") {
                    let target = parse_query_param(query, "target").unwrap_or_else(|| "default".into());
                    let req_path = parse_query_param(query, "path").unwrap_or_else(|| "/api/status".into());
                    let t0 = Instant::now();
                    match client
                        .request(&target, "GET", &req_path, HashMap::new(), Vec::new(), Duration::from_secs(12))
                        .await
                    {
                        Ok(resp) => {
                            let parsed_json: serde_json::Value = serde_json::from_slice(&resp.body)
                                .unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(&resp.body).to_string()));
                            let out = serde_json::json!({
                                "ok": true,
                                "transport": "reticulum-rs-mit-inprocess",
                                "http_sockets_on_workstation": 0,
                                "dest_hash": resp.dest_hash,
                                "status": resp.status,
                                "wire_bytes": resp.wire_bytes,
                                "logical_bytes": resp.body.len(),
                                "chunks_cht2": resp.chunks_rx,
                                "was_gzip": resp.was_gzip,
                                "latency_ms": (t0.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0,
                                "payload": parsed_json
                            });
                            responder.respond(json_response(StatusCode::OK, out.to_string()));
                        }
                        Err(e) => {
                            let out = serde_json::json!({ "ok": false, "error": e.to_string() });
                            responder.respond(json_response(StatusCode::BAD_GATEWAY, out.to_string()));
                        }
                    }
                    return;
                }
            }
            responder.respond(html_response(StatusCode::NOT_FOUND, "<h1>404 Not Found</h1>".into()));
            return;
        }

        // Native Same-Origin Reticulum Hash Dialing for Sovereign Ghost Apps (e.g. Huginn RNS):
        //   GET  /__bifrost_rns/peers                      -> list live announced Reticulum peers
        //   ANY  /__bifrost_rns/<32-hex-hash>/<sub_path>   -> dial peer directly over in-process Reticulum Link (0 CORS, 0 proxy sockets)
        if let Some(rest) = path.strip_prefix("/__bifrost_rns/") {
            if rest == "peers" || rest == "peers/" {
                let peers = client.list_peers().await;
                let payload = serde_json::json!({
                    "ok": true,
                    "self_host": host,
                    "identity": client.identity_name(),
                    "peers": peers,
                });
                responder.respond(json_response(StatusCode::OK, payload.to_string()));
                return;
            }
            let (target_hash, sub_path) = match rest.find('/') {
                Some(idx) => (&rest[..idx], &rest[idx..]),
                None => (rest, "/"),
            };
            let valid_hash = target_hash.len() == 32 && target_hash.bytes().all(|b| b.is_ascii_hexdigit());
            if !valid_hash {
                let out = serde_json::json!({
                    "ok": false,
                    "error": format!("Invalid 32-hex Reticulum destination hash: {target_hash}")
                });
                responder.respond(json_response(StatusCode::BAD_REQUEST, out.to_string()));
                return;
            }
            let target_lower = target_hash.to_ascii_lowercase();
            match client
                .request(&target_lower, &method, sub_path, hdrs, body, Duration::from_secs(25))
                .await
            {
                Ok(resp) => {
                    let st = StatusCode::from_u16(resp.status).unwrap_or(StatusCode::OK);
                    let mut builder = Response::builder()
                        .status(st)
                        .header("Access-Control-Allow-Origin", "*")
                        .header("x-bifrost-transport", "reticulum-rs-mit-inprocess-p2p")
                        .header("x-bifrost-target-hash", &target_lower);
                    let mut has_ct = false;
                    for (k, v) in resp.headers {
                        if k.eq_ignore_ascii_case("content-type") {
                            has_ct = true;
                        }
                        if let (Ok(hn), Ok(hv)) = (
                            header::HeaderName::from_bytes(k.as_bytes()),
                            header::HeaderValue::from_str(&v),
                        ) {
                            builder = builder.header(hn, hv);
                        }
                    }
                    if !has_ct {
                        builder = builder.header(header::CONTENT_TYPE, "application/json; charset=utf-8");
                    }
                    responder.respond(builder.body(resp.body).unwrap());
                }
                Err(e) => {
                    let out = serde_json::json!({
                        "ok": false,
                        "error": format!("Reticulum dial to {target_lower} failed: {e}")
                    });
                    responder.respond(json_response(StatusCode::BAD_GATEWAY, out.to_string()));
                }
            }
            return;
        }

        // Zero-DNS Local Sovereign Petname Resolution (`bifrost://forge/` -> 32-hex hash)
        let resolved_host = contacts::resolve_host_to_hash(&host);

        // Encrypted Local Immutable Asset Cache (Vite bundles, 3D twins, fonts/icons -> <15ms repeat loads)
        let cacheable = cache::is_cacheable_path(&method, &path);
        if cacheable {
            let ch_host = resolved_host.clone();
            let ch_path = path.clone();
            if let Ok(Some(hit)) = tokio::task::spawn_blocking(move || cache::get(&ch_host, &ch_path)).await {
                let st = StatusCode::from_u16(hit.status).unwrap_or(StatusCode::OK);
                let mut builder = Response::builder()
                    .status(st)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Expose-Headers", "x-bifrost-transport, x-bifrost-cache")
                    .header("x-bifrost-transport", "encrypted-local-cache")
                    .header("x-bifrost-cache", "HIT-ENC");
                let mut has_ct = false;
                for (k, v) in hit.headers {
                    if k.eq_ignore_ascii_case("content-type") {
                        has_ct = true;
                    }
                    if let (Ok(hn), Ok(hv)) = (
                        header::HeaderName::from_bytes(k.as_bytes()),
                        header::HeaderValue::from_str(&v),
                    ) {
                        builder = builder.header(hn, hv);
                    }
                }
                if !has_ct {
                    builder = builder.header(header::CONTENT_TYPE, "application/octet-stream");
                }
                responder.respond(builder.body(hit.body).unwrap());
                return;
            }
        }

        // Strip conditional cache headers: WebKitGTK incognito custom-scheme cannot satisfy empty 304 responses
        hdrs.remove("if-none-match");
        hdrs.remove("if-modified-since");

        // Live In-Process Reticulum Fetch -> 0-Port Ghost Linux Server (auto-retry once on dropped GET chunk)
        let req_timeout = if cacheable { Duration::from_secs(120) } else { Duration::from_secs(60) };
        let mut req_result = client
            .request(&resolved_host, &method, &path, hdrs.clone(), body.clone(), req_timeout)
            .await;
        if req_result.is_err() && method.eq_ignore_ascii_case("GET") {
            warn!(host = %resolved_host, path = %path, "Retrying GET over Reticulum after transient stream error");
            req_result = client
                .request(&resolved_host, &method, &path, hdrs, body, req_timeout)
                .await;
        }
        match req_result {
            Ok(resp) => {
                let is_still_encoded = resp
                    .headers
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("content-encoding"))
                    || resp.body.starts_with(&[0x1f, 0x8b]);
                if cacheable && resp.status == 200 && !is_still_encoded {
                    let put_host = resolved_host.clone();
                    let put_path = path.clone();
                    let put_status = resp.status;
                    let put_hdrs: Vec<(String, String)> = resp
                        .headers
                        .iter()
                        .filter(|(k, _)| !k.eq_ignore_ascii_case("content-encoding"))
                        .cloned()
                        .collect();
                    let put_body = resp.body.clone();
                    tokio::task::spawn_blocking(move || {
                        cache::put(&put_host, &put_path, put_status, &put_hdrs, &put_body);
                    });
                }
                let st = StatusCode::from_u16(resp.status).unwrap_or(StatusCode::OK);
                let mut builder = Response::builder()
                    .status(st)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Expose-Headers", "x-bifrost-transport, x-bifrost-cache")
                    .header("x-bifrost-transport", "reticulum-rs-mit-inprocess")
                    .header("x-bifrost-cache", if cacheable { "MISS-STORED" } else { "BYPASS" });
                let mut has_ct = false;
                for (k, v) in resp.headers {
                    if k.eq_ignore_ascii_case("content-type") {
                        has_ct = true;
                    }
                    if let (Ok(hn), Ok(hv)) = (
                        header::HeaderName::from_bytes(k.as_bytes()),
                        header::HeaderValue::from_str(&v),
                    ) {
                        builder = builder.header(hn, hv);
                    }
                }
                if !has_ct {
                    builder = builder.header(header::CONTENT_TYPE, "application/octet-stream");
                }
                responder.respond(builder.body(resp.body).unwrap());
            }
            Err(e) => {
                let relays = client.configured_peers().await;
                let relays_txt = if relays.is_empty() {
                    "none (open 📡 NODES on bifrost://home/ to add a relay)".to_string()
                } else {
                    relays.join(", ")
                };
                let err_html = format!(
                    "<html><body style='background:#020503;color:#fca5a5;font-family:ui-monospace,monospace;padding:32px;max-width:740px;margin:0 auto;'>\
                     <h2>BIFROST // RETICULUM LINK ERROR</h2>\
                     <p style='color:#fecaca;'>{e}</p>\
                     <p style='color:#94a3b8;font-size:12px;'>Connected relays: <code style='color:#7dd3fc;'>{relays_txt}</code></p>\
                     <p style='display:flex;gap:12px;margin-top:20px;'>\
                       <button onclick='window.location.reload()' style='background:rgba(132,204,22,0.18);border:1px solid #84cc16;color:#bef264;padding:8px 14px;border-radius:8px;cursor:pointer;font-family:inherit;font-weight:800;'>↻ RETRY LINK</button>\
                       <a style='color:#7dd3fc;padding:8px 14px;border:1px solid rgba(56,189,248,0.45);border-radius:8px;text-decoration:none;' href='bifrost://home/'>← Home &amp; Relays</a>\
                     </p></body></html>"
                );
                responder.respond(html_response(StatusCode::BAD_GATEWAY, err_html));
            }
        }
    });
}

fn normalize_initial_url(raw: &str) -> String {
    let v = raw.trim();
    if let Some(idx) = v.find("#ip=") {
        let rest = v[idx + 4..].trim_start_matches('/');
        if !rest.is_empty() {
            return format!("ironpass://{rest}");
        }
    }
    if v.starts_with("bifrost://") || v.starts_with("ironpass://") || v.starts_with("citadel://") {
        v.to_string()
    } else {
        "bifrost://home/".to_string()
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let client_identity = ClientIdentity::load_or_create(args.client_key_file.as_deref())?;
    if args.print_client_id {
        println!("{}", client_identity.client_id());
        return Ok(());
    }
    let ifac: Option<Arc<IfacKey>> =
        IfacKey::resolve(args.ifac_key.as_deref(), args.ifac_key_file.as_deref())?.map(Arc::new);

    let initial_url = args
        .positional_url
        .as_deref()
        .map(normalize_initial_url)
        .unwrap_or_else(|| normalize_initial_url(&args.start_url));

    let rt = Arc::new(tokio::runtime::Runtime::new()?);
    let rns_listen_opt = if args.no_listen {
        None
    } else {
        Some(args.rns_listen.clone())
    };

    // Merge CLI --rns-peer values with saved relays (~/.config/bifrost/peers.json) and persist new CLI relays
    let mut merged_peers = load_saved_relays();
    let mut added_new_cli_peer = false;
    for p in &args.rns_peers {
        let t = p.trim();
        if !t.is_empty() && !merged_peers.iter().any(|x| x == t) {
            merged_peers.push(t.to_string());
            added_new_cli_peer = true;
        }
    }
    if added_new_cli_peer {
        save_relays(&merged_peers);
    }

    let rns_client: Arc<CitadelRnsClient> = rt.block_on(CitadelRnsClient::start_with_security(
        &args.identity_name,
        rns_listen_opt.clone(),
        merged_peers,
        args.allow_hashes.clone(),
        ifac,
        client_identity,
    ));

    info!(
        start_url = %initial_url,
        web2_blocked = !args.allow_web2,
        "Launching BIFROST — The Reticulum Ghost Browser (0 local HTTP ports, RAM-only)"
    );

    let event_loop = EventLoop::new();
    let window_icon = Icon::from_rgba(ICON_BIFROST_RGBA64.to_vec(), 64, 64).ok();
    let window_title = format!(
        "BIFROST {} ({}) — Zero-IP Ghost Browser · Powered by Reticulum",
        env!("BIFROST_VERSION"),
        env!("BIFROST_GIT_SHA")
    );
    let window = WindowBuilder::new()
        .with_title(window_title)
        .with_window_icon(window_icon)
        .with_inner_size(tao::dpi::LogicalSize::new(1600.0, 960.0))
        .with_maximized(true)
        .with_resizable(true)
        .build(&event_loop)?;

    let mut web_context = WebContext::default();

    let mesh_ctx = MeshCtx {
        client: Arc::clone(&rns_client),
        listen_label: rns_listen_opt.clone(),
        allow_label: args.allow_hashes.clone(),
        llm_hash: args.llm_hash.clone(),
        llm_model: args.llm_model.clone(),
        llm_key: resolve_llm_api_key(args.llm_api_key.clone()),
    };

    let rt_bifrost = Arc::clone(&rt);
    let ctx_bifrost = mesh_ctx.clone();

    let rt_citadel = Arc::clone(&rt);
    let ctx_citadel = mesh_ctx;

    let rt_ironpass = Arc::clone(&rt);
    let client_ironpass = Arc::clone(&rns_client);

    let allow_web2 = args.allow_web2;

    let hud_script = ui::HUD_INIT_SCRIPT
        .replace("{{BIFROST_SHORT_TAG}}", env!("BIFROST_SHORT_TAG"))
        .replace("{{BIFROST_BUILD_TAG}}", env!("BIFROST_BUILD_TAG"));

    let rt_ipc = Arc::clone(&rt);
    let client_ipc = Arc::clone(&rns_client);

    let builder = WebViewBuilder::with_web_context(&mut web_context)
        .with_incognito(true)
        .with_devtools(true)
        .with_initialization_script(&hud_script)
        .with_ipc_handler(move |req: Request<String>| {
            // Only accept native IPC mutations from the local bifrost://home/ UI
            let origin_uri = req.uri().to_string();
            if !origin_uri.is_empty() && !origin_uri.starts_with("bifrost://home") && !origin_uri.starts_with("citadel://home") {
                return;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(req.body()) else {
                return;
            };
            let cmd = v.get("cmd").and_then(|c| c.as_str()).unwrap_or("");
            match cmd {
                "add_relay" => {
                    let addr = v.get("addr").and_then(|s| s.as_str()).unwrap_or("").trim().to_string();
                    if !addr.is_empty() && addr.contains(':') {
                        let mut saved = load_saved_relays();
                        if !saved.iter().any(|x| x == &addr) {
                            saved.push(addr.clone());
                            save_relays(&saved);
                        }
                        let c = Arc::clone(&client_ipc);
                        rt_ipc.spawn(async move {
                            c.connect_peer(&addr).await;
                        });
                    }
                }
                "remove_relay" => {
                    let addr = v.get("addr").and_then(|s| s.as_str()).unwrap_or("").trim().to_string();
                    if !addr.is_empty() {
                        let mut saved = load_saved_relays();
                        saved.retain(|x| x != &addr);
                        save_relays(&saved);
                    }
                }
                "save_contact" => {
                    let petname = v.get("petname").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let label = v.get("label").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let hash = v.get("hash").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let scheme = v.get("scheme").and_then(|s| s.as_str()).unwrap_or("bifrost").to_string();
                    let cpath = v.get("path").and_then(|s| s.as_str()).unwrap_or("/").to_string();
                    let notes = v.get("notes").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let card = v.get("card").and_then(|s| s.as_str());
                    let _ = upsert_contact_from_fields(petname, label, hash, scheme, cpath, notes, card);
                }
                "delete_contact" => {
                    let petname = v.get("petname").and_then(|s| s.as_str()).unwrap_or("");
                    let _ = contacts::delete_contact(petname);
                }
                "clear_cache" => {
                    let _ = cache::clear();
                }
                _ => {}
            }
        })
        .with_navigation_handler(move |url: String| {
            if url.starts_with("bifrost://")
                || url.starts_with("citadel://")
                || url.starts_with("ironpass://")
                || url == "about:blank"
                || url.starts_with("blob:bifrost://")
                || url.starts_with("blob:citadel://")
                || url.starts_with("data:")
            {
                return true;
            }
            if allow_web2 && (url.starts_with("http://") || url.starts_with("https://")) {
                return true;
            }
            warn!(blocked_url = %url, "[BIFROST-DROP] Blocked non-Reticulum egress");
            false
        })
        .with_asynchronous_custom_protocol("bifrost".into(), move |_webview_id, request, responder| {
            spawn_mesh_handler("bifrost", Arc::clone(&rt_bifrost), ctx_bifrost.clone(), request, responder);
        })
        .with_asynchronous_custom_protocol("citadel".into(), move |_webview_id, request, responder| {
            spawn_mesh_handler("citadel", Arc::clone(&rt_citadel), ctx_citadel.clone(), request, responder);
        })
        .with_asynchronous_custom_protocol("ironpass".into(), move |_webview_id, request, responder| {
            let uri = request.uri().to_string();
            let method = request.method().as_str().to_string();
            let body = request.body().clone();
            let (raw_host, path) = parse_custom_uri(&uri, "ironpass");
            let host = contacts::resolve_host_to_hash(&raw_host);
            let client = Arc::clone(&client_ironpass);

            rt_ironpass.spawn(async move {
                if method.eq_ignore_ascii_case("OPTIONS") {
                    responder.respond(cors_preflight_response());
                    return;
                }
                // Built-in Local Sandbox DPP & Live Hash-Chained IronPass Ledger (zero external servers!)
                if host.eq_ignore_ascii_case("local-demo") {
                    let clean_path = path.split('?').next().unwrap_or(&path).trim_matches('/');
                    if method.eq_ignore_ascii_case("POST") && clean_path.ends_with("/ledger") {
                        let dpp_id = clean_path
                            .strip_prefix("dpp/")
                            .unwrap_or(clean_path)
                            .strip_suffix("/ledger")
                            .unwrap_or("battery-01")
                            .trim_matches('/');
                        match sandbox::append_sandbox_ledger_event(dpp_id, &body) {
                            Ok(json_out) => responder.respond(json_response(StatusCode::OK, json_out)),
                            Err(e) => responder.respond(json_response(
                                StatusCode::BAD_REQUEST,
                                serde_json::json!({ "ok": false, "error": e }).to_string(),
                            )),
                        }
                        return;
                    }
                    let dpp_id = clean_path.strip_prefix("dpp/").unwrap_or("battery-01").trim_matches('/');
                    let t0 = Instant::now();
                    if let Some(raw_json) = sandbox::get_signed_sandbox_dpp_json(dpp_id) {
                        let latency_ms = t0.elapsed().as_secs_f64() * 1000.0;
                        let chunks = ((raw_json.len() + 399) / 400).max(1) as u32;
                        let page = dpp::render_visual_dpp(
                            &uri,
                            "local-demo",
                            &format!("/dpp/{dpp_id}"),
                            raw_json.len(),
                            raw_json.len(),
                            chunks,
                            false,
                            latency_ms,
                            &raw_json,
                        );
                        responder.respond(html_response(StatusCode::OK, page));
                    } else {
                        responder.respond(html_response(
                            StatusCode::NOT_FOUND,
                            "<h1>Sandbox DPP not found</h1><p><a href='bifrost://home/'>← Home</a></p>".into(),
                        ));
                    }
                    return;
                }

                if method.eq_ignore_ascii_case("POST") || path.trim_end_matches('/').ends_with("/ledger") {
                    let mut hdrs = HashMap::new();
                    hdrs.insert("accept".into(), "application/json".into());
                    if method.eq_ignore_ascii_case("POST") {
                        hdrs.insert("content-type".into(), "application/json".into());
                    }
                    match client
                        .request(&host, &method, &path, hdrs, body, Duration::from_secs(15))
                        .await
                    {
                        Ok(resp) => {
                            let st = StatusCode::from_u16(resp.status).unwrap_or(StatusCode::OK);
                            responder.respond(json_response(
                                st,
                                String::from_utf8_lossy(&resp.body).to_string(),
                            ));
                        }
                        Err(e) => {
                            let out = serde_json::json!({ "ok": false, "error": e.to_string() });
                            responder.respond(json_response(StatusCode::BAD_GATEWAY, out.to_string()));
                        }
                    }
                    return;
                }

                let t0 = Instant::now();
                let mut hdrs = HashMap::new();
                hdrs.insert("accept".into(), "application/json".into());
                match client
                    .request(&host, "GET", &path, hdrs, Vec::new(), Duration::from_secs(15))
                    .await
                {
                    Ok(resp) => {
                        let latency_ms = t0.elapsed().as_secs_f64() * 1000.0;
                        let page = dpp::render_visual_dpp(
                            &uri,
                            &resp.dest_hash,
                            &path,
                            resp.wire_bytes,
                            resp.body.len(),
                            resp.chunks_rx,
                            resp.was_gzip,
                            latency_ms,
                            &resp.body,
                        );
                        responder.respond(html_response(StatusCode::OK, page));
                    }
                    Err(e) => {
                        let err_html = format!(
                            "<html><body style='background:#020503;color:#fca5a5;font-family:monospace;padding:32px'>\
                             <h2>IRONPASS // DPP FETCH FAILED</h2><p>{e}</p>\
                             <p><a style='color:#84cc16' href='bifrost://home/'>← Return to bifrost://home/</a></p></body></html>"
                        );
                        responder.respond(html_response(StatusCode::BAD_GATEWAY, err_html));
                    }
                }
            });
        })
        .with_url(&initial_url);

    #[cfg(target_os = "linux")]
    let _webview = builder.build_gtk(window.default_vbox().expect("GTK vbox"))?;
    #[cfg(not(target_os = "linux"))]
    let _webview = builder.build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
