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

mod dpp;
mod ui;

use anyhow::Result;
use citadel_rns_rs::client::CitadelRnsClient;
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
#[command(version)]
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
        .body(body.into_bytes())
        .unwrap()
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
        if host.eq_ignore_ascii_case("home") {
            if path == "/" || path == "/index.html" {
                responder.respond(html_response(StatusCode::OK, ui::HOME_HTML.to_string()));
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
            if path == "/qr-sheet" || path == "/qr-sheet/" {
                let vault_hash = client
                    .list_peers()
                    .await
                    .into_iter()
                    .next()
                    .map(|p| p.hash)
                    .unwrap_or_else(|| "00000000000000000000000000000000".to_string());
                responder.respond(html_response(StatusCode::OK, dpp::render_qr_sheet_html(&vault_hash)));
                return;
            }
            if path == "/api/peers" {
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
                let payload = serde_json::json!({
                    "ok": true,
                    "identity": client.identity_name(),
                    "rns_listen": ctx.listen_label,
                    "allow_hashes": ctx.allow_label,
                    "llm_hash": detected_llm,
                    "llm_model": ctx.llm_model,
                    "bookmarks": bookmarks,
                    "peers": peers,
                });
                responder.respond(json_response(StatusCode::OK, payload.to_string()));
                return;
            }
            if path == "/api/ai-chat" && method.eq_ignore_ascii_case("POST") {
                let t0 = Instant::now();
                let req_json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        let out = serde_json::json!({ "ok": false, "error": format!("Invalid JSON body: {e}") });
                        responder.respond(json_response(StatusCode::BAD_REQUEST, out.to_string()));
                        return;
                    }
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

        // Live In-Process Reticulum Fetch -> 0-Port Ghost Linux Server
        match client
            .request(&host, &method, &path, hdrs, body, Duration::from_secs(25))
            .await
        {
            Ok(resp) => {
                let st = StatusCode::from_u16(resp.status).unwrap_or(StatusCode::OK);
                let mut builder = Response::builder()
                    .status(st)
                    .header("Access-Control-Allow-Origin", "*")
                    .header("x-bifrost-transport", "reticulum-rs-mit-inprocess");
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
                let err_html = format!(
                    "<html><body style='background:#020503;color:#fca5a5;font-family:monospace;padding:32px'>\
                     <h2>BIFROST // RETICULUM LINK ERROR</h2><p>{e}</p>\
                     <p><a style='color:#84cc16' href='bifrost://home/'>← Return to bifrost://home/</a></p></body></html>"
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

    let rns_client: Arc<CitadelRnsClient> = rt.block_on(CitadelRnsClient::start(
        &args.identity_name,
        rns_listen_opt.clone(),
        args.rns_peers.clone(),
        args.allow_hashes.clone(),
    ));

    info!(
        start_url = %initial_url,
        web2_blocked = !args.allow_web2,
        "Launching BIFROST — The Reticulum Ghost Browser (0 local HTTP ports, RAM-only)"
    );

    let event_loop = EventLoop::new();
    let window_icon = Icon::from_rgba(ICON_BIFROST_RGBA64.to_vec(), 64, 64).ok();
    let window = WindowBuilder::new()
        .with_title("BIFROST — The Reticulum Ghost Browser (bifrost:// · 0 HTTP PORTS)")
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

    let builder = WebViewBuilder::with_web_context(&mut web_context)
        .with_incognito(true)
        .with_devtools(true)
        .with_initialization_script(ui::HUD_INIT_SCRIPT)
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
            let (host, path) = parse_custom_uri(&uri, "ironpass");
            let client = Arc::clone(&client_ironpass);

            rt_ironpass.spawn(async move {
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
