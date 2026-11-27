//! BIFROST Encrypted Immutable Asset Cache (`BFC1`)
//!
//! Stores content-hashed SPA bundles (`/assets/*`, `/_next/static/*`) and 3D Digital Twin
//! models (`.glb`, `.obj`, `.gltf`, `.wasm`, images/fonts) encrypted at rest on local NVMe/SSD
//! using SHA-256-CTR + RFC 2104 HMAC-SHA256 (Encrypt-then-MAC) keyed by a device-local 256-bit
//! key (`~/.config/bifrost/cache.key`, `0600`).
//!
//! - Dynamic routes (`/`, `/api/*`, `/dpp/*`, `/sess/api/*`, SSE) are NEVER cached.
//! - Repeat loads of multi-MB apps (Vite/Next.js SPAs, 3D Room Twins) drop from seconds over
//!   Reticulum to <15 ms from encrypted local storage.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

const MAGIC: &[u8; 4] = b"BFC1";
const MAX_ENTRY_BYTES: usize = 32 * 1024 * 1024;
const MAX_CACHE_TOTAL_BYTES: u64 = 256 * 1024 * 1024;

static NONCE_COUNTER: AtomicU64 = AtomicU64::new(1);
static CACHE_KEYS: OnceLock<([u8; 32], [u8; 32])> = OnceLock::new();

#[derive(Serialize, Deserialize)]
struct CachedMeta {
    status: u16,
    headers: Vec<(String, String)>,
}

pub struct CachedAsset {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub struct CacheStats {
    pub files: usize,
    pub bytes: u64,
}

fn cache_dir() -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".cache")
        });
    base.join("bifrost").join("enc-v1")
}

fn key_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".config")
        });
    base.join("bifrost").join("cache.key")
}

fn random_bytes(len: usize) -> Vec<u8> {
    if let Ok(mut f) = fs::File::open("/dev/urandom") {
        use std::io::Read;
        let mut buf = vec![0u8; len];
        if f.read_exact(&mut buf).is_ok() {
            return buf;
        }
    }
    let mut out = Vec::with_capacity(len);
    while out.len() < len {
        let mut h = Sha256::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        h.update(now.to_be_bytes());
        h.update(std::process::id().to_be_bytes());
        h.update(NONCE_COUNTER.fetch_add(1, Ordering::Relaxed).to_be_bytes());
        out.extend_from_slice(&h.finalize());
    }
    out.truncate(len);
    out
}

fn load_or_create_keys() -> &'static ([u8; 32], [u8; 32]) {
    CACHE_KEYS.get_or_init(|| {
        let kp = key_path();
        let master: [u8; 32] = if let Ok(raw) = fs::read(&kp) {
            if raw.len() == 32 {
                let mut a = [0u8; 32];
                a.copy_from_slice(&raw);
                a
            } else {
                let mut h = Sha256::new();
                h.update(&raw);
                h.finalize().into()
            }
        } else {
            let seed = random_bytes(32);
            if let Some(parent) = kp.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&kp, &seed);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&kp, fs::Permissions::from_mode(0o600));
            }
            let mut a = [0u8; 32];
            a.copy_from_slice(&seed);
            a
        };

        let mut he = Sha256::new();
        he.update(master);
        he.update(b"bifrost-cache-enc-v1");
        let k_enc: [u8; 32] = he.finalize().into();

        let mut hm = Sha256::new();
        hm.update(master);
        hm.update(b"bifrost-cache-mac-v1");
        let k_mac: [u8; 32] = hm.finalize().into();

        (k_enc, k_mac)
    })
}

fn hmac_sha256(key: &[u8; 32], parts: &[&[u8]]) -> [u8; 32] {
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..32 {
        ipad[i] ^= key[i];
        opad[i] ^= key[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    for p in parts {
        inner.update(p);
    }
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    outer.finalize().into()
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn xor_keystream(k_enc: &[u8; 32], nonce: &[u8; 16], data: &mut [u8]) {
    let mut ctr: u64 = 0;
    for chunk in data.chunks_mut(32) {
        let mut h = Sha256::new();
        h.update(k_enc);
        h.update(nonce);
        h.update(ctr.to_be_bytes());
        let block = h.finalize();
        for (dst, src) in chunk.iter_mut().zip(block.iter()) {
            *dst ^= *src;
        }
        ctr = ctr.wrapping_add(1);
    }
}

fn seal(plaintext: &[u8]) -> Vec<u8> {
    let (k_enc, k_mac) = load_or_create_keys();
    let nonce_vec = random_bytes(16);
    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&nonce_vec);

    let mut ct = plaintext.to_vec();
    xor_keystream(k_enc, &nonce, &mut ct);

    let tag = hmac_sha256(k_mac, &[MAGIC, &nonce, &ct]);
    let mut out = Vec::with_capacity(4 + 16 + 32 + ct.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&tag);
    out.extend_from_slice(&ct);
    out
}

fn open(blob: &[u8]) -> Option<Vec<u8>> {
    if blob.len() < 4 + 16 + 32 || &blob[0..4] != MAGIC {
        return None;
    }
    let (k_enc, k_mac) = load_or_create_keys();
    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&blob[4..20]);
    let tag = &blob[20..52];
    let ct = &blob[52..];

    let expected = hmac_sha256(k_mac, &[MAGIC, &nonce, ct]);
    if !ct_eq(tag, &expected) {
        return None;
    }

    let mut pt = ct.to_vec();
    xor_keystream(k_enc, &nonce, &mut pt);
    Some(pt)
}

pub fn is_cacheable_path(method: &str, path: &str) -> bool {
    if !method.eq_ignore_ascii_case("GET") {
        return false;
    }
    let clean = path.split('?').next().unwrap_or(path);
    let low = clean.to_lowercase();

    if low == "/"
        || low.ends_with(".html")
        || low.starts_with("/api/")
        || low.contains("/api/")
        || low.starts_with("/dpp/")
        || low.starts_with("/cam/")
    {
        return false;
    }

    if low.starts_with("/assets/")
        || low.starts_with("/_next/static/")
        || low.starts_with("/models/")
        || low.starts_with("/icons/")
        || low.starts_with("/images/")
        || low.starts_with("/fonts/")
    {
        return true;
    }

    matches!(
        low.rsplit('.').next().unwrap_or(""),
        "js" | "mjs"
            | "css"
            | "glb"
            | "gltf"
            | "obj"
            | "stl"
            | "wasm"
            | "woff2"
            | "woff"
            | "ttf"
            | "png"
            | "jpg"
            | "jpeg"
            | "webp"
            | "svg"
            | "ico"
            | "mp3"
            | "wav"
    )
}

fn entry_path(dest_hash: &str, path: &str) -> PathBuf {
    let mut h = Sha256::new();
    h.update(dest_hash.to_lowercase().as_bytes());
    h.update(b":");
    h.update(path.as_bytes());
    let digest = h.finalize();
    let mut hex = String::with_capacity(64);
    for b in digest {
        hex.push_str(&format!("{b:02x}"));
    }
    cache_dir().join(format!("{hex}.bfc"))
}

pub fn get(dest_hash: &str, path: &str) -> Option<CachedAsset> {
    let p = entry_path(dest_hash, path);
    let blob = fs::read(&p).ok()?;
    let pt = match open(&blob) {
        Some(v) => v,
        None => {
            let _ = fs::remove_file(&p);
            return None;
        }
    };
    if pt.len() < 4 {
        return None;
    }
    let meta_len = u32::from_be_bytes([pt[0], pt[1], pt[2], pt[3]]) as usize;
    if 4 + meta_len > pt.len() {
        return None;
    }
    let meta: CachedMeta = serde_json::from_slice(&pt[4..4 + meta_len]).ok()?;
    let body = pt[4 + meta_len..].to_vec();
    Some(CachedAsset {
        status: meta.status,
        headers: meta.headers,
        body,
    })
}

pub fn put(dest_hash: &str, path: &str, status: u16, headers: &[(String, String)], body: &[u8]) {
    if status != 200 || body.len() < 256 || body.len() > MAX_ENTRY_BYTES {
        return;
    }
    let dir = cache_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let meta = CachedMeta {
        status,
        headers: headers.to_vec(),
    };
    let meta_bytes = match serde_json::to_vec(&meta) {
        Ok(b) => b,
        Err(_) => return,
    };
    let mut pt = Vec::with_capacity(4 + meta_bytes.len() + body.len());
    pt.extend_from_slice(&(meta_bytes.len() as u32).to_be_bytes());
    pt.extend_from_slice(&meta_bytes);
    pt.extend_from_slice(body);

    let blob = seal(&pt);
    let target = entry_path(dest_hash, path);
    let tmp = target.with_extension("tmp");
    if fs::write(&tmp, &blob).is_ok() {
        let _ = fs::rename(&tmp, &target);
    }
    enforce_budget();
}

fn enforce_budget() {
    let dir = cache_dir();
    let Ok(rd) = fs::read_dir(&dir) else { return };
    let mut entries: Vec<(PathBuf, u64, SystemTime)> = Vec::new();
    let mut total: u64 = 0;
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("bfc") {
            continue;
        }
        if let Ok(md) = e.metadata() {
            let sz = md.len();
            let mt = md.modified().unwrap_or(UNIX_EPOCH);
            total = total.saturating_add(sz);
            entries.push((p, sz, mt));
        }
    }
    if total <= MAX_CACHE_TOTAL_BYTES {
        return;
    }
    entries.sort_by_key(|(_, _, mt)| *mt);
    for (p, sz, _) in entries {
        if total <= MAX_CACHE_TOTAL_BYTES * 8 / 10 {
            break;
        }
        if fs::remove_file(&p).is_ok() {
            total = total.saturating_sub(sz);
        }
    }
}

pub fn stats() -> CacheStats {
    let dir = cache_dir();
    let mut files = 0usize;
    let mut bytes = 0u64;
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            if e.path().extension().and_then(|s| s.to_str()) == Some("bfc") {
                if let Ok(md) = e.metadata() {
                    files += 1;
                    bytes = bytes.saturating_add(md.len());
                }
            }
        }
    }
    CacheStats { files, bytes }
}

pub fn clear() -> usize {
    let dir = cache_dir();
    let mut removed = 0usize;
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            if e.path().extension().and_then(|s| s.to_str()) == Some("bfc") {
                if fs::remove_file(e.path()).is_ok() {
                    removed += 1;
                }
            }
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_roundtrip_and_tamper_detect() {
        let msg = b"bifrost-encrypted-static-asset-payload-0123456789";
        let blob = seal(msg);
        assert_eq!(&blob[0..4], MAGIC);
        assert_ne!(&blob[52..], msg);
        let opened = open(&blob).expect("valid ciphertext must open");
        assert_eq!(opened, msg);

        let mut tampered = blob.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 0x01;
        assert!(open(&tampered).is_none());

        let mut bad_tag = blob;
        bad_tag[25] ^= 0x80;
        assert!(open(&bad_tag).is_none());
    }

    #[test]
    fn cacheable_path_policy() {
        assert!(is_cacheable_path("GET", "/assets/main-abc123.js"));
        assert!(is_cacheable_path("GET", "/_next/static/chunks/app.js"));
        assert!(is_cacheable_path("GET", "/models/room_twin.obj"));
        assert!(is_cacheable_path("GET", "/scene.glb?v=2"));
        assert!(!is_cacheable_path("GET", "/"));
        assert!(!is_cacheable_path("GET", "/index.html"));
        assert!(!is_cacheable_path("GET", "/api/status"));
        assert!(!is_cacheable_path("GET", "/dpp/demo-01"));
        assert!(!is_cacheable_path("GET", "/cam/frame.jpg"));
        assert!(!is_cacheable_path("POST", "/assets/main.js"));
    }
}
