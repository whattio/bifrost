//! BIFROST Sovereign Local Contact Book (`~/.config/bifrost/contacts.json`).
//!
//! Zero-DNS, Zero-ICANN petname resolution:
//! Maps human-friendly local petnames (e.g. `forge`, `huginn`, `office`) to
//! 128-bit cryptographic Reticulum destination hashes strictly on the user's own device.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub petname: String,
    pub label: String,
    pub hash: String,
    #[serde(default = "default_scheme")]
    pub scheme: String,
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub updated_at: u64,
}

fn default_scheme() -> String {
    "bifrost".to_string()
}

fn default_path() -> String {
    "/".to_string()
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn normalize_petname(raw: &str) -> String {
    raw.trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(32)
        .collect()
}

pub fn is_valid_hash(raw: &str) -> bool {
    raw.len() == 32 && raw.bytes().all(|b| b.is_ascii_hexdigit())
}

pub fn contacts_file_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("bifrost")
        .join("contacts.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContactsFile {
    version: u32,
    contacts: Vec<Contact>,
}

pub fn load_contacts() -> Vec<Contact> {
    let path = contacts_file_path();
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(raw) => {
            if let Ok(parsed) = serde_json::from_str::<ContactsFile>(&raw) {
                return parsed.contacts;
            }
            if let Ok(list) = serde_json::from_str::<Vec<Contact>>(&raw) {
                return list;
            }
            Vec::new()
        }
        Err(_) => Vec::new(),
    }
}

pub fn save_contacts(contacts: &[Contact]) -> std::io::Result<()> {
    let path = contacts_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = ContactsFile {
        version: 1,
        contacts: contacts.to_vec(),
    };
    let json = serde_json::to_string_pretty(&file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&path, json)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub fn upsert_contact(mut contact: Contact) -> Result<Vec<Contact>, String> {
    let petname = normalize_petname(&contact.petname);
    if petname.is_empty() || petname == "home" || petname == "localhost" {
        return Err("Invalid petname (use lowercase letters, digits, '-' or '_'; cannot be 'home')".into());
    }
    let hash = contact.hash.trim().to_ascii_lowercase();
    if !is_valid_hash(&hash) {
        return Err("Destination hash must be a 32-character hex Reticulum hash".into());
    }
    contact.petname = petname.clone();
    contact.hash = hash;
    if contact.label.trim().is_empty() {
        contact.label = petname.clone();
    }
    if contact.scheme != "ironpass" && contact.scheme != "citadel" {
        contact.scheme = "bifrost".into();
    }
    if !contact.path.starts_with('/') {
        contact.path = format!("/{}", contact.path);
    }
    contact.updated_at = now_unix();

    let mut list = load_contacts();
    if let Some(existing) = list.iter_mut().find(|c| c.petname == petname) {
        *existing = contact;
    } else {
        list.push(contact);
    }
    list.sort_by(|a, b| a.petname.cmp(&b.petname));
    save_contacts(&list).map_err(|e| format!("Failed saving contacts.json: {e}"))?;
    Ok(list)
}

pub fn delete_contact(petname: &str) -> Result<Vec<Contact>, String> {
    let key = normalize_petname(petname);
    let mut list = load_contacts();
    list.retain(|c| c.petname != key);
    save_contacts(&list).map_err(|e| format!("Failed saving contacts.json: {e}"))?;
    Ok(list)
}

/// Resolves a host segment (`forge`, `huginn`, or a raw 32-hex hash) into its 32-hex Reticulum hash.
/// Purely local lookup — 0 DNS, 0 network calls.
pub fn resolve_host_to_hash(host: &str) -> String {
    let trimmed = host.trim().to_ascii_lowercase();
    if is_valid_hash(&trimmed) {
        return trimmed;
    }
    let key = normalize_petname(&trimmed);
    if key.is_empty() {
        return trimmed;
    }
    for c in load_contacts() {
        if c.petname == key {
            return c.hash;
        }
    }
    trimmed
}
