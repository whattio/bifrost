//! BIFROST 1-QR / 1-Link Realm Pairing (`bifrost-pair://...`)
//!
//! Bundles this device's Reticulum relays, Ghost Dock bookmarks, and saved zero-DNS
//! server petnames into a single compact offline token & scannable QR SVG so any new
//! Mac or Linux BIFROST joins your Ghost Realm in 5 seconds (0 cloud, 0 DNS).

use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::contacts;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairBookmark {
    pub label: String,
    pub uri: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub tone: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairBundle {
    pub v: u8,
    #[serde(default)]
    pub r: Vec<String>,
    /// Compact node tuples: [petname, label, hash, scheme, path]
    #[serde(default)]
    pub n: Vec<[String; 5]>,
    /// Compact dock tuples: [label, uri, icon]
    #[serde(default)]
    pub b: Vec<[String; 3]>,
}

fn base64url_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
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
        }
        if i + 2 < input.len() {
            out.push(TABLE[(triple & 0x3f) as usize] as char);
        }
        i += 3;
    }
    out
}

fn b64_val(c: u8) -> Option<u32> {
    match c {
        b'A'..=b'Z' => Some((c - b'A') as u32),
        b'a'..=b'z' => Some((c - b'a' + 26) as u32),
        b'0'..=b'9' => Some((c - b'0' + 52) as u32),
        b'-' | b'+' => Some(62),
        b'_' | b'/' => Some(63),
        _ => None,
    }
}

fn base64url_decode(raw: &str) -> Result<Vec<u8>, String> {
    let cleaned: Vec<u8> = raw
        .bytes()
        .filter(|b| !b.is_ascii_whitespace() && *b != b'=')
        .collect();
    if cleaned.len() % 4 == 1 {
        return Err("Invalid base64url length".into());
    }
    let mut out = Vec::with_capacity((cleaned.len() * 3) / 4);
    let mut i = 0;
    while i < cleaned.len() {
        let a = b64_val(cleaned[i]).ok_or_else(|| "Invalid base64url character".to_string())?;
        let b = b64_val(cleaned[i + 1]).ok_or_else(|| "Invalid base64url character".to_string())?;
        out.push(((a << 2) | (b >> 4)) as u8);
        if i + 2 < cleaned.len() {
            let c = b64_val(cleaned[i + 2]).ok_or_else(|| "Invalid base64url character".to_string())?;
            out.push((((b & 0x0f) << 4) | (c >> 2)) as u8);
            if i + 3 < cleaned.len() {
                let d = b64_val(cleaned[i + 3]).ok_or_else(|| "Invalid base64url character".to_string())?;
                out.push((((c & 0x03) << 6) | d) as u8);
            }
        }
        i += 4;
    }
    Ok(out)
}

pub fn bookmarks_file_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("bifrost").join("bookmarks.json")
}

pub fn load_bookmarks() -> Vec<PairBookmark> {
    let p = bookmarks_file_path();
    if let Ok(contents) = std::fs::read_to_string(p) {
        if let Ok(items) = serde_json::from_str::<Vec<PairBookmark>>(&contents) {
            return items;
        }
    }
    Vec::new()
}

pub fn save_bookmarks(items: &[PairBookmark]) -> Result<(), String> {
    let p = bookmarks_file_path();
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create config dir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(items).map_err(|e| format!("Serialize error: {e}"))?;
    std::fs::write(&p, json).map_err(|e| format!("Write error: {e}"))
}

pub fn encode_pair_token(
    relays: &[String],
    saved_contacts: &[contacts::Contact],
    bookmarks: &[PairBookmark],
) -> String {
    let mut r_clean: Vec<String> = Vec::new();
    for r in relays {
        let t = r.trim();
        if !t.is_empty() && t.contains(':') && !r_clean.iter().any(|x| x == t) {
            r_clean.push(t.to_string());
        }
    }
    let n_clean: Vec<[String; 5]> = saved_contacts
        .iter()
        .filter(|c| contacts::is_valid_hash(&c.hash))
        .take(10)
        .map(|c| {
            [
                c.petname.clone(),
                c.label.clone(),
                c.hash.to_ascii_lowercase(),
                if c.scheme.is_empty() { "bifrost".into() } else { c.scheme.clone() },
                if c.path.is_empty() { "/".into() } else { c.path.clone() },
            ]
        })
        .collect();
    let b_clean: Vec<[String; 3]> = bookmarks
        .iter()
        .take(8)
        .map(|b| [b.label.clone(), b.uri.clone(), b.icon.clone()])
        .collect();

    let bundle = PairBundle {
        v: 1,
        r: r_clean,
        n: n_clean,
        b: b_clean,
    };
    let raw = serde_json::to_vec(&bundle).unwrap_or_default();
    format!("bifrost-pair://{}", base64url_encode(&raw))
}

pub fn decode_pair_token(raw: &str) -> Result<PairBundle, String> {
    let trimmed = raw.trim();
    let payload = trimmed
        .strip_prefix("bifrost-pair://")
        .or_else(|| trimmed.strip_prefix("BIFROST-PAIR://"))
        .unwrap_or(trimmed)
        .trim_end_matches('/');
    if payload.is_empty() {
        return Err("Empty pairing link".into());
    }
    let bytes = base64url_decode(payload)?;
    let bundle: PairBundle =
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid pairing payload: {e}"))?;
    if bundle.v != 1 {
        return Err(format!("Unsupported pairing version: {}", bundle.v));
    }
    Ok(bundle)
}

pub fn render_pair_qr_svg(token: &str) -> String {
    match QrCode::with_error_correction_level(token.as_bytes(), EcLevel::L) {
        Ok(code) => code
            .render::<svg::Color>()
            .min_dimensions(240, 240)
            .quiet_zone(true)
            .dark_color(svg::Color("#020805"))
            .light_color(svg::Color("#ffffff"))
            .build(),
        Err(_) => "<svg width='240' height='240'></svg>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_pair_token_and_qr() {
        let relays = vec!["192.0.2.10:14295".to_string(), "198.51.100.20:14290".to_string()];
        let nodes = vec![contacts::Contact {
            petname: "forge".into(),
            label: "IronPass Forge".into(),
            hash: "0123456789abcdef0123456789abcdef".into(),
            scheme: "bifrost".into(),
            path: "/".into(),
            notes: "".into(),
            updated_at: 0,
        }];
        let bms = vec![PairBookmark {
            label: "Forge".into(),
            uri: "bifrost://0123456789abcdef0123456789abcdef/".into(),
            icon: "⚒️".into(),
            tone: "".into(),
        }];
        let token = encode_pair_token(&relays, &nodes, &bms);
        assert!(token.starts_with("bifrost-pair://"));
        let decoded = decode_pair_token(&token).expect("valid token");
        assert_eq!(decoded.v, 1);
        assert_eq!(decoded.r, relays);
        assert_eq!(decoded.n.len(), 1);
        assert_eq!(decoded.n[0][0], "forge");
        assert_eq!(decoded.b.len(), 1);
        assert_eq!(decoded.b[0][0], "Forge");
        let svg = render_pair_qr_svg(&token);
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn rejects_corrupt_pair_token() {
        assert!(decode_pair_token("bifrost-pair://not-valid-json").is_err());
        assert!(decode_pair_token("").is_err());
    }
}
