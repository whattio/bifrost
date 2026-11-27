//! BIFROST — Built-in Sovereign Start Page (Google-Style Omnibox + Local AI) & Floating HUD.
//! Served strictly from Rust memory on `bifrost://home/` (0 network, 0 HTTP sockets).
//! Zero hardcoded private hashes: dynamically discovers connected Reticulum peers and optionally
//! loads operator bookmarks from `~/.config/bifrost/bookmarks.json`.

pub const HOME_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>BIFROST — Zero-IP Ghost Browser · Powered by Reticulum</title>
<link rel="icon" type="image/png" href="bifrost://home/icons/bifrost.png" />
<style>
  :root {
    --bg: #050907;
    --surface: rgba(11, 20, 15, 0.90);
    --surface2: rgba(15, 28, 21, 0.94);
    --surface-hover: rgba(24, 42, 32, 0.96);
    --lime: #84cc16;
    --emerald: #10b981;
    --holo: #38bdf8;
    --amber: #f59e0b;
    --violet: #a78bfa;
    --danger: #f87171;
    --text: #f3faf6;
    --muted: #8ea99a;
    --border: rgba(148, 198, 168, 0.18);
  }
  * { box-sizing: border-box; margin: 0; padding: 0; }
  html, body { height: 100%; }
  body {
    min-height: 100vh;
    background:
      radial-gradient(ellipse 68% 42% at 50% 18%, rgba(132, 204, 22, 0.09), transparent 65%),
      radial-gradient(ellipse 50% 36% at 85% 90%, rgba(56, 189, 248, 0.07), transparent 60%),
      var(--bg);
    color: var(--text);
    font-family: system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
    display: flex;
    flex-direction: column;
    overflow-x: hidden;
  }
  .mono { font-family: ui-monospace, 'JetBrains Mono', SFMono-Regular, Menlo, monospace; }

  .topbar {
    width: 100%; padding: 14px 22px;
    display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap;
  }
  .top-left, .top-right { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .status-chip {
    display: inline-flex; align-items: center; gap: 7px; padding: 5px 11px; border-radius: 999px;
    font-size: 11px; font-weight: 700; letter-spacing: 0.04em;
    background: rgba(12, 22, 16, 0.85); border: 1px solid var(--border); color: var(--muted);
  }
  .status-chip.online { border-color: rgba(132,204,22,0.42); background: rgba(132,204,22,0.11); color: #d9f99d; }
  .status-chip.waiting { border-color: rgba(245,158,11,0.42); background: rgba(245,158,11,0.10); color: #fde68a; }
  .dot { width: 7px; height: 7px; border-radius: 50%; background: var(--amber); box-shadow: 0 0 8px var(--amber); }
  .status-chip.online .dot { background: var(--lime); box-shadow: 0 0 9px var(--lime); }

  .top-btn {
    display: inline-flex; align-items: center; gap: 6px;
    background: rgba(12, 22, 16, 0.85); color: #d4e8dc; border: 1px solid var(--border);
    border-radius: 999px; padding: 6px 13px; font-size: 11.5px; font-weight: 700;
    cursor: pointer; transition: all 0.15s ease; text-decoration: none;
  }
  .top-btn:hover { background: var(--surface-hover); border-color: rgba(132,204,22,0.45); color: #fff; }
  .top-btn.active { border-color: rgba(167,139,250,0.6); background: rgba(167,139,250,0.16); color: #ddd6fe; }

  .quickbar {
    width: 100%; padding: 0 22px 10px;
    display: flex; align-items: center; justify-content: center; gap: 7px; flex-wrap: wrap;
  }
  .qlink {
    display: inline-flex; align-items: center; gap: 5px; padding: 5px 11px; border-radius: 999px;
    font-size: 11px; font-weight: 700; text-decoration: none; color: #d4e8dc;
    background: rgba(11, 22, 16, 0.88); border: 1px solid rgba(148,198,168,0.22);
    cursor: pointer; transition: all 0.14s ease;
  }
  .qlink:hover { background: rgba(132,204,22,0.16); border-color: rgba(132,204,22,0.52); color: #ecfccb; }
  .qlink.holo { border-color: rgba(56,189,248,0.34); color: #bae6fd; }
  .qlink.amber { border-color: rgba(245,158,11,0.34); color: #fde68a; }

  .stage {
    flex: 1; width: 100%; max-width: 820px; margin: 0 auto; padding: 0 22px 32px;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
  }
  .stage.chat-active { justify-content: flex-start; padding-top: 8px; }

  .hero-brand {
    display: flex; flex-direction: column; align-items: center; gap: 10px;
    margin-bottom: 28px; text-align: center;
  }
  .stage.chat-active .hero-brand {
    flex-direction: row; margin-bottom: 14px; width: 100%; justify-content: space-between;
  }
  .brand-lockup { display: flex; align-items: center; gap: 14px; }
  .wordmark { font-size: 46px; font-weight: 800; letter-spacing: 0.16em; color: #f4fbf7; line-height: 1; }
  .stage.chat-active .wordmark { font-size: 19px; }
  .tagline { font-size: 12px; letter-spacing: 0.14em; color: var(--muted); font-weight: 600; }
  .stage.chat-active .tagline { display: none; }
  .new-chat-btn {
    display: none; background: rgba(132,204,22,0.14); color: var(--lime);
    border: 1px solid rgba(132,204,22,0.45); border-radius: 999px;
    padding: 6px 13px; font-size: 11.5px; font-weight: 800; cursor: pointer;
  }
  .stage.chat-active .new-chat-btn { display: inline-flex; align-items: center; gap: 6px; }

  .omnibox-wrap { width: 100%; max-width: 740px; }
  .stage.chat-active .omnibox-wrap { max-width: 100%; }
  .omnibox {
    display: flex; align-items: center; gap: 10px;
    background: rgba(11, 20, 15, 0.94); border: 1.5px solid rgba(148,198,168,0.28);
    border-radius: 999px; padding: 7px 9px 7px 18px;
    box-shadow: 0 12px 38px rgba(0,0,0,0.45); transition: border-color 0.18s ease, box-shadow 0.18s ease;
  }
  .omnibox:focus-within { border-color: rgba(132,204,22,0.68); box-shadow: 0 14px 44px rgba(0,0,0,0.55), 0 0 28px rgba(132,204,22,0.16); }
  .omnibox.mode-uri:focus-within { border-color: rgba(56,189,248,0.72); }
  .omni-icon { font-size: 18px; color: var(--lime); width: 22px; text-align: center; flex-shrink: 0; }
  .omnibox.mode-uri .omni-icon { color: var(--holo); }
  .omnibox input { flex: 1; background: transparent; border: none; outline: none; color: #fff; font-size: 15.5px; padding: 9px 4px; min-width: 0; }
  .omnibox input::placeholder { color: #6f8b7b; }
  .omni-submit {
    border: none; border-radius: 999px; padding: 10px 18px; font-size: 12.5px; font-weight: 800;
    cursor: pointer; background: linear-gradient(135deg, #84cc16, #10b981); color: #031108; flex-shrink: 0;
  }
  .omnibox.mode-uri .omni-submit { background: linear-gradient(135deg, #38bdf8, #0ea5e9); color: #02121c; }
  .omni-hint { margin-top: 10px; text-align: center; font-size: 11.5px; color: #6b8777; }
  .stage.chat-active .omni-hint { display: none; }

  .shortcuts {
    margin-top: 26px; display: flex; flex-wrap: wrap; justify-content: center; gap: 14px; max-width: 740px;
  }
  .stage.chat-active .shortcuts { display: none; }
  .shortcut {
    width: 104px; padding: 12px 6px 10px; border-radius: 14px;
    display: flex; flex-direction: column; align-items: center; gap: 8px;
    cursor: pointer; background: transparent; border: 1px solid transparent;
    color: #cfe4d7; text-decoration: none; transition: all 0.15s ease;
  }
  .shortcut:hover { background: rgba(16,31,23,0.85); border-color: var(--border); color: #fff; }
  .sc-circle {
    width: 50px; height: 50px; border-radius: 50%; display: grid; place-items: center;
    font-size: 20px; background: rgba(14,27,20,0.96); border: 1px solid rgba(148,198,168,0.22);
  }
  .sc-label { font-size: 11.5px; font-weight: 600; text-align: center; line-height: 1.25; }

  .chat-thread { display: none; width: 100%; margin-top: 18px; flex-direction: column; gap: 14px; padding-bottom: 24px; }
  .stage.chat-active .chat-thread { display: flex; }
  .msg { display: flex; flex-direction: column; gap: 6px; max-width: 100%; }
  .msg.user { align-items: flex-end; }
  .msg.assistant, .msg.error { align-items: flex-start; }
  .bubble { padding: 13px 17px; border-radius: 16px; font-size: 14.5px; line-height: 1.6; max-width: 92%; word-break: break-word; }
  .msg.user .bubble { background: rgba(132,204,22,0.16); border: 1px solid rgba(132,204,22,0.42); color: #f4fee8; }
  .msg.assistant .bubble { background: var(--surface); border: 1px solid var(--border); color: #ecf7f0; width: 100%; }
  .msg.error .bubble { background: rgba(127,29,29,0.25); border: 1px solid rgba(248,113,113,0.45); color: #fecaca; }
  .bubble code { font-family: ui-monospace, monospace; font-size: 12.5px; background: rgba(2,8,5,0.85); padding: 1px 5px; border-radius: 5px; color: #bae6fd; }
  .provenance { display: flex; flex-wrap: wrap; gap: 8px; font-size: 10.5px; color: #7ea18d; }
  .provenance span { background: rgba(10,20,14,0.85); border: 1px solid rgba(148,198,168,0.16); border-radius: 999px; padding: 2px 8px; }
  .provenance span.hi { color: #bef264; border-color: rgba(132,204,22,0.32); }

  .drawer-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.58); backdrop-filter: blur(4px); display: none; justify-content: flex-end; z-index: 100; }
  .drawer-backdrop.open { display: flex; }
  .drawer { width: min(560px, 94vw); height: 100%; background: #060d09; border-left: 1px solid rgba(132,204,22,0.35); padding: 20px 22px; overflow-y: auto; display: flex; flex-direction: column; gap: 16px; }
  .drawer-head { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: 12px; }
  .peer-card { background: var(--surface2); border: 1px solid var(--border); border-radius: 12px; padding: 13px 14px; display: flex; flex-direction: column; gap: 8px; }
  .hash-box { background: rgba(2,7,4,0.92); border: 1px solid rgba(56,189,248,0.25); border-radius: 7px; padding: 5px 8px; font-size: 11px; color: #bae6fd; word-break: break-all; }
  .stats-row { display: flex; justify-content: space-between; font-size: 11px; color: var(--muted); }
  .card-actions { display: flex; flex-wrap: wrap; gap: 7px; }
  .btn-sm { background: rgba(132,204,22,0.15); color: var(--lime); border: 1px solid rgba(132,204,22,0.45); border-radius: 7px; padding: 6px 11px; font-size: 11px; font-weight: 800; cursor: pointer; }
  .btn-sm.amber { background: rgba(245,158,11,0.13); color: #fcd34d; border-color: rgba(245,158,11,0.42); }
</style>
</head>
<body>
  <header class="topbar mono">
    <div class="top-left">
      <span class="status-chip waiting" id="npu-chip">
        <span class="dot"></span>
        <span id="npu-status-text">LOCAL AI PEER · OPTIONAL</span>
      </span>
      <span class="status-chip">0 HTTP PORTS</span>
      <span class="status-chip" id="ifac-chip">IFAC: OFF</span>
      <span class="status-chip" style="border-color:rgba(56,189,248,0.48);background:rgba(56,189,248,0.12);color:#bae6fd;" title="Synced with GitHub Cargo.toml + Git commit SHA">{{BIFROST_BUILD_TAG}}</span>
    </div>
    <div class="top-right">
      <button type="button" class="top-btn mono" id="think-toggle" onclick="toggleThinking()">🧠 THINK: OFF</button>
      <button type="button" class="top-btn mono" onclick="openQrScanner()" style="border-color:rgba(132,204,22,0.55);color:#bef264;">📷 SCAN QR</button>
      <a class="top-btn mono" href="bifrost://home/qr-sheet">🖨️ QR TAGS</a>
      <button type="button" class="top-btn mono" id="contacts-btn" onclick="openContacts()" title="Local Sovereign Contact Book (~/.config/bifrost/contacts.json · 0 DNS)">📒 CONTACTS (0)</button>
      <button type="button" class="top-btn mono" id="nodes-btn" onclick="openDrawer()">📡 NODES (0)</button>
    </div>
  </header>

  <nav class="quickbar mono" id="quickbar" aria-label="Direct Reticulum Links"></nav>

  <main class="stage" id="stage">
    <div class="hero-brand">
      <div class="brand-lockup">
        <img src="bifrost://home/icons/bifrost.png" alt="BIFROST" style="width:62px;height:62px;border-radius:16px;box-shadow:0 0 28px rgba(56,189,248,0.28);border:1px solid rgba(132,204,22,0.42);" />
        <div class="wordmark mono">BIFROST</div>
        <span class="mono" style="padding:4px 10px;border-radius:999px;font-size:11px;font-weight:800;letter-spacing:0.06em;background:rgba(56,189,248,0.14);border:1px solid rgba(56,189,248,0.5);color:#7dd3fc;">{{BIFROST_SHORT_TAG}}</span>
      </div>
      <div class="tagline mono">ZERO-IP GHOST BROWSER · POWERED BY RETICULUM · 0 COOKIES · 0 DNS</div>
      <div class="mono" style="max-width:780px;margin:8px auto 0;padding:7px 14px;border-radius:10px;background:rgba(16,185,129,0.08);border:1px solid rgba(16,185,129,0.28);color:#a7f3d0;font-size:11px;line-height:1.45;">
        🔒 <strong>Encryption:</strong> Every link is end-to-end encrypted with <strong>512-bit Curve25519</strong> (<strong>X25519</strong> ephemeral key exchange + <strong>Ed25519</strong> signatures) and <strong>AES-256-CBC + HMAC-SHA256</strong> over Reticulum — zero TLS CAs, zero DNS, zero open server ports.
      </div>
      <button type="button" class="new-chat-btn mono" onclick="resetChat()">✦ NEW CHAT</button>
    </div>

    <div class="omnibox-wrap">
      <form class="omnibox" id="omnibox-form" onsubmit="handleOmnibox(event)">
        <span class="omni-icon" id="omni-icon">🔒</span>
        <input
          id="omni-input"
          autocomplete="off"
          autofocus
          placeholder="Enter bifrost://<hash>/, ironpass://<hash>/dpp/..., Hybrid #ip= QR, or ask your local AI peer..."
          oninput="updateOmniboxMode()"
        />
        <button type="submit" class="omni-submit mono" id="omni-submit">GO ↵</button>
      </form>
      <div class="omni-hint mono">
        Zero-IP Reticulum browsing over <code>bifrost://</code> &amp; <code>ironpass://</code> · Connect any 0-port <code>citadel-vault-daemon</code> peer or local Ollama/llama.cpp AI node.
      </div>
    </div>

    <div class="shortcuts" id="shortcuts">
      <a class="shortcut" href="bifrost://home/showcase">
        <div class="sc-circle"><img src="bifrost://home/icons/citadel.png" alt="Microsite Example" style="width:44px;height:44px;border-radius:12px;" /></div>
        <div class="sc-label">Example Microsite</div>
      </a>
      <a class="shortcut" href="ironpass://local-demo/dpp/battery-01">
        <div class="sc-circle"><img src="bifrost://home/icons/ironpass.png" alt="Battery DPP" style="width:44px;height:44px;border-radius:12px;" /></div>
        <div class="sc-label">Battery DPP + Ledger</div>
      </a>
      <a class="shortcut" href="ironpass://local-demo/dpp/drone-01">
        <div class="sc-circle">🛸</div>
        <div class="sc-label">UAV MRO DPP</div>
      </a>
      <a class="shortcut" href="ironpass://local-demo/dpp/furniture-01">
        <div class="sc-circle">🪑</div>
        <div class="sc-label">ESPR Desk DPP</div>
      </a>
      <a class="shortcut" href="bifrost://home/qr-sheet">
        <div class="sc-circle">🖨️</div>
        <div class="sc-label">QR Tag Sheet</div>
      </a>
      <button type="button" class="shortcut" onclick="openDrawer()">
        <div class="sc-circle">📡</div>
        <div class="sc-label">Ghost Nodes</div>
      </button>
    </div>

    <section class="chat-thread" id="chat-thread" aria-live="polite"></section>
  </main>

  <div class="drawer-backdrop" id="drawer-backdrop" onclick="if(event.target===this) closeDrawer()">
    <aside class="drawer">
      <div class="drawer-head">
        <h2 class="mono" style="font-size:15px;">📡 DISCOVERED RETICULUM NODES</h2>
        <button type="button" class="top-btn mono" onclick="closeDrawer()">✕ CLOSE</button>
      </div>
      <div class="mono" style="font-size:11px; color:var(--muted);" id="listen-info">Scanning Reticulum announces...</div>
      <div class="peer-card" style="border-color:rgba(56,189,248,0.38);">
        <div class="mono" style="font-size:11px;font-weight:800;color:#7dd3fc;">🌐 RETICULUM RELAYS (SAVED TO ~/.config/bifrost/peers.json)</div>
        <div id="relays-list" class="mono" style="font-size:11px;color:var(--muted);">None configured yet.</div>
        <form onsubmit="event.preventDefault(); addRelayPeer();" style="display:flex;gap:6px;">
          <input id="relay-addr-input" class="mono" placeholder="IP:port or host:port (e.g. 203.0.113.10:14290)" style="flex:1;background:rgba(2,8,5,0.92);border:1px solid var(--border);color:#fff;border-radius:8px;padding:6px 9px;font-size:11px;outline:none;" />
          <button type="submit" class="btn-sm mono">＋ CONNECT</button>
        </form>
        <div id="relay-msg" class="mono" style="font-size:10.5px;color:var(--muted);"></div>
      </div>
      <div id="peers-list" style="display:flex; flex-direction:column; gap:10px;"></div>
    </aside>
  </div>

  <div class="drawer-backdrop" id="contacts-backdrop" onclick="if(event.target===this) closeContacts()">
    <aside class="drawer">
      <div class="drawer-head">
        <h2 class="mono" style="font-size:15px;">📒 SOVEREIGN CONTACT BOOK</h2>
        <button type="button" class="top-btn mono" onclick="closeContacts()">✕ CLOSE</button>
      </div>
      <div class="mono" style="font-size:11px; color:var(--muted);">
        Zero-DNS local petnames stored on this device (<code>~/.config/bifrost/contacts.json</code>). Type <code>bifrost://&lt;petname&gt;/</code> or paste a <code>bifrost-card://</code> invite.
      </div>
      <div class="peer-card" style="border-color:rgba(56,189,248,0.42);">
        <div style="display:flex;justify-content:space-between;align-items:center;gap:8px;">
          <div class="mono" style="font-size:11px;font-weight:800;color:#7dd3fc;">🔑 MY DEVICE ED25519 CLIENT ID (LOCK 2)</div>
          <button type="button" class="btn-sm mono" onclick="copyMyClientId()">📋 COPY ID</button>
        </div>
        <div class="hash-box mono" id="my-client-id">loading...</div>
        <div class="mono" style="font-size:10px;color:var(--muted);">Share this ID with a private vault owner (<code>allowed_clients.txt</code>) to authorize this device.</div>
      </div>
      <div class="peer-card" style="border-color:rgba(132,204,22,0.38);">
        <div class="mono" style="font-size:11px; font-weight:800; color:var(--lime);">＋ ADD CONTACT OR PASTE INVITE CARD</div>
        <input id="cb-card" class="mono" placeholder="Paste bifrost-card://... or bifrost://<hash>/" oninput="parseInviteCardInput()" style="background:rgba(2,8,5,0.92);border:1px solid var(--border);color:#fff;border-radius:8px;padding:7px 9px;font-size:11px;outline:none;" />
        <div style="display:grid; grid-template-columns:1fr 1.4fr; gap:7px;">
          <input id="cb-petname" class="mono" placeholder="petname (e.g. forge)" style="background:rgba(2,8,5,0.92);border:1px solid var(--border);color:#fff;border-radius:8px;padding:7px 9px;font-size:11.5px;outline:none;" />
          <input id="cb-label" placeholder="Friendly label" style="background:rgba(2,8,5,0.92);border:1px solid var(--border);color:#fff;border-radius:8px;padding:7px 9px;font-size:11.5px;outline:none;" />
        </div>
        <input id="cb-hash" class="mono" placeholder="32-hex 128-bit Reticulum destination hash" style="background:rgba(2,8,5,0.92);border:1px solid var(--border);color:#bae6fd;border-radius:8px;padding:7px 9px;font-size:11px;outline:none;" />
        <input id="cb-notes" placeholder="Notes (optional)" style="background:rgba(2,8,5,0.92);border:1px solid var(--border);color:#fff;border-radius:8px;padding:7px 9px;font-size:11px;outline:none;" />
        <div style="display:flex; align-items:center; justify-content:space-between; gap:8px;">
          <span id="cb-msg" class="mono" style="font-size:10.5px; color:var(--muted);">Ready.</span>
          <button type="button" class="btn-sm mono" onclick="saveContact()">💾 SAVE CONTACT</button>
        </div>
      </div>
      <div id="contacts-list" style="display:flex; flex-direction:column; gap:10px;"></div>
    </aside>
  </div>

  <div class="drawer-backdrop" id="qr-backdrop" onclick="if(event.target===this) closeQrScanner()">
    <aside class="drawer">
      <div class="drawer-head">
        <h2 class="mono" style="font-size:15px;">📷 IRONPASS HYBRID QR SCANNER</h2>
        <button type="button" class="top-btn mono" onclick="closeQrScanner()">✕ CLOSE</button>
      </div>
      <div class="mono" style="font-size:11px; color:var(--muted);">
        100% offline local QR decoding (0 cloud calls). Hold a printed <strong>Hybrid #ip=</strong>, <strong>ironpass://</strong>, or <strong>bifrost-card://</strong> tag to your camera, drop/paste a QR screenshot, or scan with a USB scanner.
      </div>
      <div class="peer-card" style="border-color:rgba(132,204,22,0.45); align-items:center;">
        <div style="position:relative; width:100%; max-width:380px; aspect-ratio:4/3; background:#020604; border:1px solid rgba(132,204,22,0.35); border-radius:10px; overflow:hidden; display:flex; align-items:center; justify-content:center;">
          <video id="qr-video" autoplay playsinline muted style="width:100%; height:100%; object-fit:cover; display:none;"></video>
          <div id="qr-placeholder" class="mono" style="font-size:11px; color:var(--muted); text-align:center; padding:16px;">Camera idle — click START CAMERA below or drop/upload a QR image.</div>
          <div style="position:absolute; inset:16%; border:2px dashed rgba(132,204,22,0.6); border-radius:10px; pointer-events:none;"></div>
        </div>
        <canvas id="qr-canvas" style="display:none;"></canvas>
        <div style="display:flex; flex-wrap:wrap; gap:8px; width:100%; justify-content:center;">
          <button type="button" class="btn-sm mono" id="qr-cam-btn" onclick="toggleQrCamera()">🎥 START CAMERA</button>
          <label class="btn-sm mono" style="display:inline-flex; align-items:center; gap:4px; cursor:pointer; background:rgba(56,189,248,0.14); color:#bae6fd; border-color:rgba(56,189,248,0.45);">
            🖼️ UPLOAD QR IMAGE
            <input type="file" id="qr-file-input" accept="image/*" style="display:none;" onchange="decodeQrImageFile(event)" />
          </label>
        </div>
        <div id="qr-status" class="mono" style="font-size:11px; color:var(--lime); text-align:center; word-break:break-all;">Ready to scan.</div>
      </div>
      <div class="peer-card" style="border-color:rgba(56,189,248,0.38);">
        <div class="mono" style="font-size:11px; font-weight:800; color:#7dd3fc;">⌨️ USB QR SCANNER / PASTE HYBRID #ip= LINK</div>
        <form onsubmit="event.preventDefault(); submitQrManual();" style="display:flex; gap:6px;">
          <input id="qr-manual-input" class="mono" placeholder="Scan with USB reader or paste https://...#ip=..." style="flex:1; background:rgba(2,8,5,0.92); border:1px solid var(--border); color:#fff; border-radius:8px; padding:7px 9px; font-size:11px; outline:none;" />
          <button type="submit" class="btn-sm mono">GO →</button>
        </form>
      </div>
    </aside>
  </div>

<script src="/js/jsqr.js"></script>
<script>
const conversation = [];
let isBusy = false;
let enableThinking = false;
let llmHash = null;
let llmModel = 'qwen3-4b-npu';
let savedContacts = [];
let latestPeers = [];
let myClientId = '';
async function copyTextSafe(text) {
  if (!text) return false;
  try {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch (_) {}
  try {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.setAttribute('readonly', '');
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.focus();
    ta.select();
    const ok = document.execCommand('copy');
    ta.remove();
    return Boolean(ok);
  } catch (_) {
    return false;
  }
}

async function copyMyClientId() {
  if (!myClientId) return;
  const m = document.getElementById('cb-msg');
  const ok = await copyTextSafe(myClientId);
  if (m) {
    m.style.color = 'var(--lime)';
    m.textContent = ok ? 'Copied your Ed25519 Device ID!' : `Device ID: ${myClientId}`;
  }
}

function findContactByPetname(raw) {
  const key = (raw || '').trim().toLowerCase().replace(/^@/, '');
  return savedContacts.find(c => c.petname === key) || null;
}
function findContactByHash(hash) {
  const h = (hash || '').trim().toLowerCase();
  return savedContacts.find(c => c.hash === h) || null;
}

function detectGhostUri(raw) {
  const v = (raw || '').trim();
  if (!v) return null;
  if (/^bifrost-card:\/\//i.test(v)) return v;
  const hybrid = v.match(/#ip=([^\s]+)/i);
  if (hybrid && hybrid[1]) return 'ironpass://' + hybrid[1].replace(/^\/+/, '');
  if (/^(bifrost|citadel|ironpass):\/\//i.test(v)) return v;
  if (/^[0-9a-f]{32}(\/.*)?$/i.test(v)) return 'bifrost://' + (v.includes('/') ? v : v + '/');
  const [head, ...rest] = v.split('/');
  const matched = findContactByPetname(head);
  if (matched) {
    const scheme = matched.scheme || 'bifrost';
    const sub = rest.length ? '/' + rest.join('/') : (matched.path || '/');
    return `${scheme}://${matched.petname}${sub}`;
  }
  return null;
}

function updateOmniboxMode() {
  const input = document.getElementById('omni-input');
  const form = document.getElementById('omnibox-form');
  const icon = document.getElementById('omni-icon');
  const btn = document.getElementById('omni-submit');
  const detected = detectGhostUri(input.value);
  const isCard = Boolean(detected && detected.startsWith('bifrost-card://'));
  const isUri = Boolean(detected);
  form.classList.toggle('mode-uri', isUri);
  icon.textContent = isCard ? '📒' : (isUri ? '🔒' : '✦');
  if (!isBusy) btn.textContent = isCard ? 'IMPORT CARD →' : (isUri ? 'OPEN URI →' : 'ASK AI ↵');
}

function toggleThinking() {
  enableThinking = !enableThinking;
  const btn = document.getElementById('think-toggle');
  btn.textContent = enableThinking ? '🧠 THINK: ON' : '🧠 THINK: OFF';
  btn.classList.toggle('active', enableThinking);
}

function escHtml(s) {
  return String(s || '').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}

function appendMessage(role, content, meta = null) {
  document.getElementById('stage').classList.add('chat-active');
  const thread = document.getElementById('chat-thread');
  const wrap = document.createElement('div');
  wrap.className = `msg ${role}`;
  const bubble = document.createElement('div');
  bubble.className = 'bubble';
  bubble.innerHTML = escHtml(content).replace(/\n/g, '<br/>');
  wrap.appendChild(bubble);
  if (role === 'assistant' && meta) {
    const prov = document.createElement('div');
    prov.className = 'provenance mono';
    const shortHash = (meta.dest_hash || '').slice(0, 8) + '…' + (meta.dest_hash || '').slice(-4);
    prov.innerHTML = `<span class="hi">⚡ ${escHtml(meta.model || llmModel)}</span><span>${((meta.latency_ms||0)/1000).toFixed(2)}s</span><span>${meta.wire_bytes||0} B encrypted</span><span>RNS ${shortHash}</span><span>0 Cloud Calls</span>`;
    wrap.appendChild(prov);
  }
  thread.appendChild(wrap);
  window.scrollTo({ top: document.body.scrollHeight, behavior: 'smooth' });
}

function resetChat() {
  conversation.length = 0;
  document.getElementById('chat-thread').innerHTML = '';
  document.getElementById('stage').classList.remove('chat-active');
  document.getElementById('omni-input').focus();
}

async function handleOmnibox(e) {
  if (e && e.preventDefault) e.preventDefault();
  const input = document.getElementById('omni-input');
  const raw = input.value.trim();
  if (!raw || isBusy) return;
  const uri = detectGhostUri(raw);
  if (uri && uri.startsWith('bifrost-card://')) {
    input.value = '';
    updateOmniboxMode();
    openContacts();
    document.getElementById('cb-card').value = uri;
    parseInviteCardInput();
    return;
  }
  if (uri) { window.location.href = uri; return; }

  input.value = '';
  appendMessage('user', raw);
  conversation.push({ role: 'user', content: raw });
  isBusy = true;
  document.getElementById('omni-submit').disabled = true;

  try {
    const bodyStr = JSON.stringify({ model: llmModel, enable_thinking: enableThinking, messages: conversation.slice(-12) });
    const encoded = encodeURIComponent(bodyStr);
    const r = encoded.length <= 6000
      ? await fetch(`/api/ai-chat?payload=${encoded}&_t=${Date.now()}`)
      : await fetch('/api/ai-chat', {
          method: 'POST',
          headers: { 'Content-Type': 'text/plain;charset=UTF-8' },
          body: bodyStr,
        });
    const d = await r.json();
    if (!d.ok) appendMessage('error', d.error || 'Local AI peer unavailable.');
    else { conversation.push({ role: 'assistant', content: d.reply || '' }); appendMessage('assistant', d.reply || '', d); }
  } catch (err) {
    appendMessage('error', `Reticulum transport error: ${err}`);
  } finally {
    isBusy = false;
    document.getElementById('omni-submit').disabled = false;
    updateOmniboxMode();
  }
}

function openDrawer() { document.getElementById('drawer-backdrop').classList.add('open'); refreshPeers(); }
function closeDrawer() { document.getElementById('drawer-backdrop').classList.remove('open'); }

let qrStream = null;
let qrScanTimer = null;

function openQrScanner() {
  document.getElementById('qr-backdrop').classList.add('open');
  const st = document.getElementById('qr-status');
  if (st) { st.style.color = 'var(--lime)'; st.textContent = 'Ready — click START CAMERA, upload/paste a QR image (Ctrl+V / Cmd+V), or scan via USB.'; }
  setTimeout(() => { const m = document.getElementById('qr-manual-input'); if (m) m.focus(); }, 60);
}

function stopQrCamera() {
  if (qrScanTimer) { clearInterval(qrScanTimer); qrScanTimer = null; }
  if (qrStream) {
    try { qrStream.getTracks().forEach(t => t.stop()); } catch (_) {}
    qrStream = null;
  }
  const v = document.getElementById('qr-video');
  const ph = document.getElementById('qr-placeholder');
  const btn = document.getElementById('qr-cam-btn');
  if (v) { v.srcObject = null; v.style.display = 'none'; }
  if (ph) ph.style.display = 'block';
  if (btn) btn.textContent = '🎥 START CAMERA';
}

function closeQrScanner() {
  stopQrCamera();
  document.getElementById('qr-backdrop').classList.remove('open');
}

function handleDecodedQrPayload(raw) {
  const txt = (raw || '').trim();
  const st = document.getElementById('qr-status');
  if (!txt) return false;
  const hybrid = txt.match(/#ip=([^\s]+)/i);
  if (hybrid && hybrid[1]) {
    const dest = 'ironpass://' + hybrid[1].replace(/^\/+/, '');
    if (st) { st.style.color = 'var(--lime)'; st.textContent = `✓ HYBRID #ip= DETECTED — Stripping HTTPS locally → ${dest}`; }
    stopQrCamera();
    setTimeout(() => { window.location.href = dest; }, 350);
    return true;
  }
  if (/^bifrost-card:\/\//i.test(txt)) {
    closeQrScanner();
    openContacts();
    document.getElementById('cb-card').value = txt;
    parseInviteCardInput();
    return true;
  }
  if (/^(ironpass|bifrost|citadel):\/\//i.test(txt)) {
    if (st) { st.style.color = 'var(--lime)'; st.textContent = `✓ OPENING ZERO-IP URI → ${txt}`; }
    stopQrCamera();
    setTimeout(() => { window.location.href = txt; }, 250);
    return true;
  }
  if (/^[0-9a-f]{32}(\/.*)?$/i.test(txt)) {
    const dest = 'bifrost://' + (txt.includes('/') ? txt : txt + '/');
    if (st) { st.style.color = 'var(--lime)'; st.textContent = `✓ 128-BIT HASH → ${dest}`; }
    stopQrCamera();
    setTimeout(() => { window.location.href = dest; }, 250);
    return true;
  }
  if (st) { st.style.color = 'var(--amber)'; st.textContent = `Decoded non-Reticulum QR (blocked by Ghost isolation): ${txt}`; }
  return false;
}

function submitQrManual() {
  const inp = document.getElementById('qr-manual-input');
  const val = (inp.value || '').trim();
  if (!val) return;
  if (handleDecodedQrPayload(val)) inp.value = '';
}

async function toggleQrCamera() {
  if (qrStream) { stopQrCamera(); return; }
  const st = document.getElementById('qr-status');
  const v = document.getElementById('qr-video');
  const ph = document.getElementById('qr-placeholder');
  const btn = document.getElementById('qr-cam-btn');
  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    if (st) { st.style.color = 'var(--amber)'; st.textContent = 'WebView camera API unavailable on custom scheme — use 🖼️ UPLOAD QR IMAGE, paste a screenshot (Ctrl+V/Cmd+V), or USB scanner!'; }
    return;
  }
  if (st) { st.style.color = 'var(--muted)'; st.textContent = 'Requesting local webcam...'; }
  try {
    qrStream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'environment' }, audio: false });
    v.srcObject = qrStream;
    v.style.display = 'block';
    if (ph) ph.style.display = 'none';
    if (btn) btn.textContent = '⏹ STOP CAMERA';
    if (st) { st.style.color = 'var(--lime)'; st.textContent = 'Camera LIVE — hold printed Hybrid #ip= or ironpass:// QR inside the frame...'; }
    const canvas = document.getElementById('qr-canvas');
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    qrScanTimer = setInterval(() => {
      if (!v || v.readyState < 2 || !window.jsQR) return;
      canvas.width = v.videoWidth || 640;
      canvas.height = v.videoHeight || 480;
      ctx.drawImage(v, 0, 0, canvas.width, canvas.height);
      const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const code = window.jsQR(imgData.data, imgData.width, imgData.height, { inversionAttempts: 'attemptBoth' });
      if (code && code.data) {
        handleDecodedQrPayload(code.data);
      }
    }, 150);
  } catch (err) {
    if (st) { st.style.color = 'var(--amber)'; st.textContent = `Camera blocked (${err.message || err}) — use 🖼️ UPLOAD QR IMAGE or paste a screenshot (Ctrl+V / Cmd+V)!`; }
  }
}

function decodeQrFromBlob(blob) {
  const st = document.getElementById('qr-status');
  if (!blob || !window.jsQR) return;
  const url = URL.createObjectURL(blob);
  const img = new Image();
  img.onload = () => {
    try {
      const canvas = document.getElementById('qr-canvas');
      const ctx = canvas.getContext('2d', { willReadFrequently: true });
      canvas.width = img.naturalWidth || img.width;
      canvas.height = img.naturalHeight || img.height;
      ctx.drawImage(img, 0, 0);
      const imgData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const code = window.jsQR(imgData.data, imgData.width, imgData.height, { inversionAttempts: 'attemptBoth' });
      if (code && code.data) {
        handleDecodedQrPayload(code.data);
      } else if (st) {
        st.style.color = 'var(--danger)';
        st.textContent = 'No QR code detected in that image — try a tighter crop or higher contrast.';
      }
    } finally {
      URL.revokeObjectURL(url);
    }
  };
  img.src = url;
}

function decodeQrImageFile(ev) {
  const f = ev.target && ev.target.files && ev.target.files[0];
  if (f) decodeQrFromBlob(f);
  if (ev.target) ev.target.value = '';
}

window.addEventListener('paste', (ev) => {
  const bd = document.getElementById('qr-backdrop');
  if (!bd || !bd.classList.contains('open')) return;
  const items = (ev.clipboardData && ev.clipboardData.items) || [];
  for (const it of items) {
    if (it.type && it.type.startsWith('image/')) {
      const blob = it.getAsFile();
      if (blob) { ev.preventDefault(); decodeQrFromBlob(blob); return; }
    }
  }
});
function openContacts(prefillHash = '', prefillLabel = '') {
  document.getElementById('contacts-backdrop').classList.add('open');
  if (prefillHash) {
    document.getElementById('cb-hash').value = prefillHash;
    const existing = findContactByHash(prefillHash);
    if (existing) {
      document.getElementById('cb-petname').value = existing.petname || '';
      document.getElementById('cb-label').value = existing.label || '';
      document.getElementById('cb-notes').value = existing.notes || '';
    } else {
      document.getElementById('cb-label').value = prefillLabel || '';
      const slug = (prefillLabel || prefillHash.slice(0, 8)).toLowerCase().replace(/[^a-z0-9-_]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 20);
      document.getElementById('cb-petname').value = slug;
      document.getElementById('cb-notes').value = '';
    }
    document.getElementById('cb-msg').textContent = 'Review petname & click Save.';
  }
  refreshContacts();
}
function closeContacts() { document.getElementById('contacts-backdrop').classList.remove('open'); }
function parseInviteCardInput() {
  const raw = (document.getElementById('cb-card').value || '').trim();
  if (!raw) return;
  if (raw.startsWith('bifrost-card://')) {
    const body = raw.slice('bifrost-card://'.length);
    const [mainPart, qPart] = body.split('?');
    const idPart = mainPart.split('/')[0] || '';
    if (idPart.includes('@')) {
      const [p, h] = idPart.split('@');
      if (p) document.getElementById('cb-petname').value = p;
      if (h) document.getElementById('cb-hash').value = h;
    } else if (/^[0-9a-f]{32}$/i.test(idPart)) {
      document.getElementById('cb-hash').value = idPart.toLowerCase();
    }
    if (qPart) {
      const params = new URLSearchParams(qPart);
      if (params.get('label')) document.getElementById('cb-label').value = params.get('label');
      if (params.get('notes')) document.getElementById('cb-notes').value = params.get('notes');
    }
    document.getElementById('cb-msg').textContent = 'Invite card parsed — click Save!';
    return;
  }
  const m = raw.match(/([0-9a-f]{32})/i);
  if (m) {
    document.getElementById('cb-hash').value = m[1].toLowerCase();
    document.getElementById('cb-msg').textContent = '32-hex hash extracted — enter a petname & click Save.';
  }
}
async function refreshContacts() {
  try {
    const r = await fetch(`/api/contacts?_t=${Date.now()}`);
    const d = await r.json();
    savedContacts = d.contacts || [];
    const btn = document.getElementById('contacts-btn');
    if (btn) btn.textContent = `📒 CONTACTS (${savedContacts.length})`;
    renderContactsList();
  } catch (e) {}
}
function makeInviteCard(c) {
  const params = new URLSearchParams();
  if (c.label) params.set('label', c.label);
  if (c.notes) params.set('notes', c.notes);
  if (c.scheme && c.scheme !== 'bifrost') params.set('scheme', c.scheme);
  const pathPart = c.path && c.path !== '/' ? c.path : '';
  return `bifrost-card://${c.petname}@${c.hash}${pathPart}?${params.toString()}`;
}
async function copyInviteCard(petname) {
  const c = findContactByPetname(petname);
  if (!c) return;
  const card = makeInviteCard(c);
  const msg = document.getElementById('cb-msg');
  const ok = await copyTextSafe(card);
  if (msg) {
    msg.style.color = 'var(--lime)';
    msg.textContent = ok ? `Copied invite card for @${c.petname}!` : card;
  }
}
async function saveContact() {
  const msg = document.getElementById('cb-msg');
  if ((document.getElementById('cb-card').value || '').trim() && !(document.getElementById('cb-hash').value || '').trim()) {
    parseInviteCardInput();
  }
  const params = new URLSearchParams();
  const card = (document.getElementById('cb-card').value || '').trim();
  const petname = (document.getElementById('cb-petname').value || '').trim();
  const label = (document.getElementById('cb-label').value || '').trim();
  const hash = (document.getElementById('cb-hash').value || '').trim();
  const notes = (document.getElementById('cb-notes').value || '').trim();
  if (card) params.set('card', card);
  if (petname) params.set('petname', petname);
  if (label) params.set('label', label);
  if (hash) params.set('hash', hash);
  if (notes) params.set('notes', notes);
  params.set('_t', String(Date.now()));
  msg.style.color = 'var(--muted)';
  msg.textContent = 'Saving...';
  try {
    // Plain same-origin GET (no custom headers / no cache option -> zero WebKit CORS preflight on macOS)
    const r = await fetch(`/api/contacts/save?${params.toString()}`);
    const d = await r.json();
    if (!d.ok) { msg.style.color = 'var(--danger)'; msg.textContent = d.error || 'Could not save contact.'; return; }
    savedContacts = d.contacts || [];
    document.getElementById('contacts-btn').textContent = `📒 CONTACTS (${savedContacts.length})`;
    ['cb-card','cb-petname','cb-label','cb-hash','cb-notes'].forEach(id => document.getElementById(id).value = '');
    msg.style.color = 'var(--lime)';
    msg.textContent = `Saved locally (${savedContacts.length} contacts in ~/.config/bifrost/contacts.json).`;
    renderContactsList();
    refreshPeers();
  } catch (e) {
    // Native Wry IPC fallback (zero HTTP / zero fetch)
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify({ cmd: 'save_contact', card, petname, label, hash, notes }));
      ['cb-card','cb-petname','cb-label','cb-hash','cb-notes'].forEach(id => document.getElementById(id).value = '');
      msg.style.color = 'var(--lime)';
      msg.textContent = 'Saved via native IPC (~/.config/bifrost/contacts.json).';
      setTimeout(() => { refreshContacts(); refreshPeers(); }, 200);
    } else {
      msg.style.color = 'var(--danger)';
      msg.textContent = `Save error: ${e}`;
    }
  }
}
async function deleteContact(petname) {
  try {
    const r = await fetch(`/api/contacts/delete?petname=${encodeURIComponent(petname)}&_t=${Date.now()}`);
    const d = await r.json();
    if (d.ok) {
      savedContacts = d.contacts || [];
      document.getElementById('contacts-btn').textContent = `📒 CONTACTS (${savedContacts.length})`;
      renderContactsList();
      refreshPeers();
    }
  } catch (e) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify({ cmd: 'delete_contact', petname }));
      setTimeout(() => { refreshContacts(); refreshPeers(); }, 200);
    } else {
      const msg = document.getElementById('cb-msg');
      if (msg) { msg.style.color = 'var(--danger)'; msg.textContent = `Delete error: ${e}`; }
    }
  }
}
function renderContactsList() {
  const el = document.getElementById('contacts-list');
  if (!el) return;
  if (!savedContacts.length) {
    el.innerHTML = `<div class="peer-card"><div class="mono" style="font-size:12px;color:var(--muted);">No saved contacts yet. Paste an invite card above or click "+ SAVE CONTACT" on any peer in NODES.</div></div>`;
    return;
  }
  const activeHashes = new Set((latestPeers || []).filter(p => p.active).map(p => p.hash));
  el.innerHTML = savedContacts.map(c => {
    const online = activeHashes.has(c.hash);
    const petUri = `${c.scheme || 'bifrost'}://${c.petname}${c.path || '/'}`;
    return `
      <div class="peer-card">
        <div style="display:flex; justify-content:space-between; align-items:center; gap:8px;">
          <h3 style="font-size:14px;">${escHtml(c.label || c.petname)}</h3>
          <span class="status-chip ${online ? 'online' : 'waiting'} mono" style="padding:3px 8px;font-size:10px;">${online ? 'ONLINE' : 'SAVED'}</span>
        </div>
        <div class="hash-box mono" style="color:#d9f99d;">${escHtml(petUri)} → ${escHtml(c.hash.slice(0, 12))}…${escHtml(c.hash.slice(-6))}</div>
        ${c.notes ? `<div style="font-size:11px; color:var(--muted);">${escHtml(c.notes)}</div>` : ''}
        <div class="card-actions">
          <button class="btn-sm mono" onclick="window.location.href='${escHtml(petUri)}'">OPEN ${escHtml('@' + c.petname)}</button>
          <button class="btn-sm mono" onclick="copyInviteCard('${escHtml(c.petname)}')">📋 COPY CARD</button>
          <button class="btn-sm amber mono" onclick="deleteContact('${escHtml(c.petname)}')">✕</button>
        </div>
      </div>
    `;
  }).join('');
}

async function addRelayPeer() {
  const inp = document.getElementById('relay-addr-input');
  const msg = document.getElementById('relay-msg');
  const addr = (inp.value || '').trim();
  if (!addr) return;
  if (!addr.includes(':')) {
    msg.style.color = 'var(--danger)';
    msg.textContent = 'Include port, e.g. 203.0.113.10:14290';
    return;
  }
  msg.style.color = 'var(--muted)';
  msg.textContent = `Dialing ${addr}...`;
  try {
    const r = await fetch(`/api/relays/add?addr=${encodeURIComponent(addr)}&_t=${Date.now()}`);
    const d = await r.json();
    if (!d.ok) { msg.style.color = 'var(--danger)'; msg.textContent = d.error || 'Could not add relay.'; return; }
    inp.value = '';
    msg.style.color = 'var(--lime)';
    msg.textContent = `Connected & saved ${addr} — waiting for peer announces (~2s)...`;
    refreshPeers();
  } catch (e) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify({ cmd: 'add_relay', addr }));
      inp.value = '';
      msg.style.color = 'var(--lime)';
      msg.textContent = `Connected & saved ${addr} via native IPC — waiting for peers (~2s)...`;
      setTimeout(refreshPeers, 400);
    } else {
      msg.style.color = 'var(--danger)';
      msg.textContent = `Relay error: ${e}`;
    }
  }
}

async function removeRelayPeer(addr) {
  try {
    await fetch(`/api/relays/remove?addr=${encodeURIComponent(addr)}&_t=${Date.now()}`);
    const msg = document.getElementById('relay-msg');
    if (msg) { msg.style.color = 'var(--amber)'; msg.textContent = `Removed ${addr} from saved relays (takes effect next launch).`; }
    refreshPeers();
  } catch (_) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify({ cmd: 'remove_relay', addr }));
      setTimeout(refreshPeers, 250);
    }
  }
}

async function refreshPeers() {
  try {
    const r = await fetch(`/api/peers?_t=${Date.now()}`);
    const d = await r.json();
    llmHash = d.llm_hash || null;
    if (d.llm_model) llmModel = d.llm_model;
    if (d.client_id) {
      myClientId = d.client_id;
      const cidEl = document.getElementById('my-client-id');
      if (cidEl) cidEl.textContent = d.client_id;
    }
    const relaysEl = document.getElementById('relays-list');
    if (relaysEl) {
      const relays = d.relays || [];
      relaysEl.innerHTML = relays.length
        ? relays.map(x => `<span style="display:inline-flex;align-items:center;gap:6px;margin:2px 6px 2px 0;padding:2px 8px;border-radius:999px;background:rgba(56,189,248,0.12);border:1px solid rgba(56,189,248,0.38);color:#bae6fd;">${escHtml(x)} <button type="button" onclick="removeRelayPeer('${escHtml(x)}')" style="background:none;border:none;color:#fca5a5;cursor:pointer;font-size:10px;padding:0;">✕</button></span>`).join('')
        : 'No outbound relays configured. Add your relay host:port below.';
    }
    const ifacEl = document.getElementById('ifac-chip');
    if (ifacEl) {
      if (d.ifac_fingerprint) {
        ifacEl.className = 'status-chip online';
        ifacEl.textContent = `🔒 IFAC: ${d.ifac_fingerprint}`;
      } else {
        ifacEl.className = 'status-chip waiting';
        ifacEl.textContent = 'IFAC: OFF';
      }
    }
    const peers = d.peers || [];
    latestPeers = peers;
    renderContactsList();
    const bookmarks = d.bookmarks || [];

    document.getElementById('nodes-btn').textContent = `📡 NODES (${peers.length})`;
    document.getElementById('listen-info').textContent = `Identity: ${d.identity} · Listener: ${d.rns_listen || 'outbound-only'}`;

    const npuPeer = llmHash ? peers.find(p => p.hash === llmHash && p.active) : null;
    const chip = document.getElementById('npu-chip');
    const txt = document.getElementById('npu-status-text');
    if (npuPeer) {
      chip.className = 'status-chip online';
      txt.textContent = `LOCAL AI · ${llmModel.toUpperCase()} ONLINE`;
    } else {
      chip.className = 'status-chip waiting';
      txt.textContent = `LOCAL AI PEER · ${peers.length ? 'OPTIONAL' : 'WAITING FOR PEERS'}`;
    }

    // Dynamic quickbar from ~/.config/bifrost/bookmarks.json or discovered peers
    const qb = document.getElementById('quickbar');
    if (bookmarks.length) {
      qb.innerHTML = bookmarks.map(b => `<a class="qlink ${escHtml(b.tone||'')}" href="${escHtml(b.uri)}">${escHtml(b.icon||'🔒')} ${escHtml(b.label)}</a>`).join('');
    } else if (peers.length) {
      qb.innerHTML = peers.map(p => `<a class="qlink holo" href="bifrost://${p.hash}/">🔒 ${escHtml(p.alias)}</a><a class="qlink amber" href="ironpass://${p.hash}/dpp/demo-01">📜 Demo DPP</a>`).join('');
    } else {
      qb.innerHTML = `<a class="qlink holo" href="bifrost://home/showcase">🌐 Example Microsite</a><a class="qlink amber" href="ironpass://local-demo/dpp/battery-01">🔋 Battery DPP + Ledger</a><a class="qlink amber" href="ironpass://local-demo/dpp/drone-01">🛸 UAV DPP</a><a class="qlink" href="bifrost://home/qr-sheet">🖨️ QR Sheet</a>`;
    }

    const list = document.getElementById('peers-list');
    if (!peers.length) {
      list.innerHTML = `<div class="peer-card"><div class="mono" style="font-size:12px;color:var(--muted);">No 0-port Ghost peers announced yet.<br/><br/>Start the local demo vault:<br/><code>python3 examples/demo-vault/server.py</code><br/><code>citadel-vault-daemon --upstream http://127.0.0.1:8080 --rns-peer 127.0.0.1:14290</code></div></div>`;
      return;
    }

    list.innerHTML = peers.map(p => {
      const saved = findContactByHash(p.hash);
      const displayName = saved ? `${saved.label} (@${saved.petname})` : p.alias;
      const target = saved ? escHtml(saved.petname) : p.hash;
      return `
      <div class="peer-card">
        <div style="display:flex; justify-content:space-between; align-items:center; gap:8px;">
          <h3 style="font-size:14px;">${escHtml(displayName)}</h3>
          <span class="status-chip ${p.active ? 'online' : 'waiting'} mono" style="padding:3px 8px;font-size:10px;">${p.active ? 'X25519 LIVE' : 'LINKING'}</span>
        </div>
        <div class="hash-box mono">bifrost://${p.hash}/</div>
        <div class="stats-row mono">
          <span>Reqs: ${p.requests}</span>
          <span>Wire: ${(p.wire_bytes/1024).toFixed(1)} KB</span>
          <span>Chunks: ${p.chunks_rx}</span>
        </div>
        <div class="card-actions">
          <button class="btn-sm mono" onclick="window.location.href='bifrost://${target}/'">OPEN GHOST SITE →</button>
          <button class="btn-sm amber mono" onclick="window.location.href='ironpass://${target}/dpp/demo-01'">OPEN DEMO DPP + LEDGER</button>
          <button class="btn-sm mono" onclick="closeDrawer(); openContacts('${escHtml(p.hash)}', '${escHtml(p.alias)}')">${saved ? '✎ EDIT CONTACT' : '＋ SAVE CONTACT'}</button>
        </div>
      </div>
    `;
    }).join('');
  } catch (err) {
    document.getElementById('peers-list').innerHTML = `<div class="peer-card"><div class="mono" style="font-size:12px;color:var(--danger);">Error: ${escHtml(err)}</div></div>`;
  }
}

refreshContacts();
refreshPeers();
setInterval(refreshPeers, 3000);
</script>
</body>
</html>
"##;

pub const HUD_INIT_SCRIPT: &str = r#"
(function() {
  function ensureStorage(name) {
    try {
      const s = window[name];
      s.setItem('__bifrost_probe', '1');
      s.removeItem('__bifrost_probe');
    } catch (_) {
      const mem = Object.create(null);
      const shim = {
        getItem: (k) => (Object.prototype.hasOwnProperty.call(mem, String(k)) ? mem[String(k)] : null),
        setItem: (k, v) => { mem[String(k)] = String(v); },
        removeItem: (k) => { delete mem[String(k)]; },
        clear: () => { for (const k in mem) delete mem[k]; },
        key: (i) => Object.keys(mem)[i] ?? null,
        get length() { return Object.keys(mem).length; },
      };
      try { Object.defineProperty(window, name, { value: shim, configurable: true }); } catch (__) {}
    }
  }
  ensureStorage('localStorage');
  ensureStorage('sessionStorage');

  if (window.__bifrostHudMounted) return;
  if (window.location.protocol !== 'bifrost:' && window.location.protocol !== 'citadel:' && window.location.protocol !== 'ironpass:') return;
  if ((window.location.protocol === 'bifrost:' || window.location.protocol === 'citadel:') && window.location.hostname === 'home') return;
  window.__bifrostHudMounted = true;

  function mountHud() {
    if (!document.body || document.getElementById('__bifrost_ghost_hud')) return;
    const bar = document.createElement('div');
    bar.id = '__bifrost_ghost_hud';
    bar.style.cssText = [
      'position:fixed', 'top:0', 'left:0', 'right:0', 'height:34px', 'z-index:2147483647',
      'background:rgba(2,8,5,0.95)', 'border-bottom:1px solid rgba(132,204,22,0.45)',
      'display:flex', 'align-items:center', 'gap:8px', 'padding:0 10px',
      'font-family:ui-monospace,SFMono-Regular,Menlo,monospace', 'font-size:11.5px', 'color:#d9f99d',
      'box-shadow:0 4px 16px rgba(0,0,0,0.55)'
    ].join(';');

    bar.innerHTML = `
      <button id="__bf_home" style="background:rgba(132,204,22,0.18);border:1px solid rgba(132,204,22,0.55);color:#bef264;border-radius:6px;padding:3px 9px;cursor:pointer;font-weight:800;font-family:inherit;font-size:11px;white-space:nowrap;">ᛉ BIFROST</button>
      <span title="{{BIFROST_BUILD_TAG}}" style="padding:2px 7px;border-radius:999px;background:rgba(56,189,248,0.14);border:1px solid rgba(56,189,248,0.45);color:#7dd3fc;font-size:10px;font-weight:800;white-space:nowrap;">{{BIFROST_SHORT_TAG}}</span>
      <button id="__bf_back" style="background:transparent;border:1px solid rgba(16,185,129,0.4);color:#a7f3d0;border-radius:6px;padding:3px 7px;cursor:pointer;font-family:inherit;">←</button>
      <button id="__bf_reload" style="background:transparent;border:1px solid rgba(16,185,129,0.4);color:#a7f3d0;border-radius:6px;padding:3px 7px;cursor:pointer;font-family:inherit;">↻</button>
      <input id="__bf_addr" style="flex:1;min-width:120px;background:rgba(0,0,0,0.65);border:1px solid rgba(56,189,248,0.4);color:#e0f2fe;border-radius:6px;padding:4px 9px;font-family:inherit;font-size:11.5px;outline:none;" />
      <span style="padding:3px 8px;border-radius:999px;background:rgba(16,185,129,0.16);border:1px solid rgba(16,185,129,0.45);color:#6ee7b7;font-size:10px;font-weight:800;white-space:nowrap;">🔒 X25519+Ed25519 · 0 HTTP PORTS</span>
    `;
    document.body.prepend(bar);
    document.body.style.paddingTop = '34px';

    const addr = document.getElementById('__bf_addr');
    addr.value = window.location.href;
    addr.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        let v = addr.value.trim();
        const hybrid = v.match(/#ip=([^\s]+)/i);
        if (hybrid && hybrid[1]) v = 'ironpass://' + hybrid[1].replace(/^\/+/, '');
        else if (v && !v.includes('://')) v = 'bifrost://' + v;
        if (v) window.location.href = v;
      }
    });
    document.getElementById('__bf_home').onclick = () => { window.location.href = 'bifrost://home/'; };
    document.getElementById('__bf_back').onclick = () => window.history.back();
    document.getElementById('__bf_reload').onclick = () => window.location.reload();
  }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', mountHud);
  else mountHud();
})();
"#;
