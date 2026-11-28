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
    --agent-color: #84cc16;
    --agent-glow: rgba(132, 204, 22, 0.22);
    --agent-soft: rgba(132, 204, 22, 0.14);
    flex: 1; width: 100%; max-width: 820px; margin: 0 auto; padding: 0 22px 24px;
    display: flex; flex-direction: column; align-items: center; justify-content: center;
  }
  .stage.chat-active { justify-content: flex-start; padding-top: 8px; }

  @keyframes orbitSpin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
  .council-strip {
    display: flex; align-items: center; justify-content: center; gap: 14px;
    margin-bottom: 18px; flex-wrap: wrap; transition: all 0.22s ease;
  }
  .stage.chat-active .council-strip {
    gap: 8px; margin-bottom: 10px; width: 100%; justify-content: flex-start;
  }
  .bot-medallion {
    position: relative; display: flex; flex-direction: column; align-items: center; gap: 6px;
    padding: 8px 12px 7px; border-radius: 15px;
    background: rgba(8, 17, 13, 0.56); border: 1px solid rgba(148, 198, 168, 0.12);
    color: #93b0a0; cursor: pointer; opacity: 0.68;
    transition: transform 0.2s ease, opacity 0.2s ease, border-color 0.22s ease, background 0.22s ease, box-shadow 0.22s ease;
  }
  .stage.chat-active .bot-medallion {
    flex-direction: row; padding: 4px 10px 4px 5px; border-radius: 999px; gap: 7px;
  }
  .bot-medallion:hover {
    opacity: 0.96; transform: translateY(-2px);
    border-color: var(--m-color, rgba(132, 204, 22, 0.48)); color: #fff;
  }
  .bot-medallion.active {
    opacity: 1; transform: translateY(-2px);
    background: radial-gradient(circle at 50% 30%, var(--m-soft, rgba(132, 204, 22, 0.16)), rgba(6, 15, 11, 0.92));
    border-color: var(--m-color, #84cc16);
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.52), 0 0 22px var(--m-glow, rgba(132, 204, 22, 0.24));
    color: #fff;
  }
  .bot-orb {
    position: relative; width: 50px; height: 50px; border-radius: 50%; padding: 2px;
    background: rgba(4, 11, 8, 0.96); border: 1.5px solid rgba(148, 198, 168, 0.24);
    transition: all 0.2s ease;
  }
  .stage.chat-active .bot-orb { width: 24px; height: 24px; padding: 1px; }
  .bot-medallion.active .bot-orb {
    border-color: var(--m-color, #84cc16);
    box-shadow: 0 0 16px var(--m-glow, rgba(132, 204, 22, 0.35));
  }
  .bot-medallion.active .bot-orb::after {
    content: ''; position: absolute; inset: -5px; border-radius: 50%;
    border: 1px dashed var(--m-color, #84cc16); opacity: 0.58;
    animation: orbitSpin 12s linear infinite; pointer-events: none;
  }
  .stage.chat-active .bot-medallion.active .bot-orb::after { display: none; }
  .bot-orb img {
    width: 100%; height: 100%; border-radius: 50%; object-fit: cover; display: block;
    filter: saturate(0.8) brightness(0.88); transition: filter 0.2s ease;
  }
  .bot-medallion:hover .bot-orb img, .bot-medallion.active .bot-orb img { filter: saturate(1.1) brightness(1.04); }
  .bot-name { font-size: 11px; font-weight: 800; letter-spacing: 0.08em; line-height: 1; }
  .bot-role { font-size: 9.5px; color: #759685; letter-spacing: 0.03em; line-height: 1; }
  .bot-medallion.active .bot-role { color: #d1fae5; }
  .stage.chat-active .bot-role { display: none; }

  .msg-agent-head {
    display: flex; align-items: center; gap: 8px;
    margin-bottom: 8px; padding-bottom: 7px; border-bottom: 1px solid rgba(148, 198, 168, 0.14);
  }
  .msg-agent-head img {
    width: 24px; height: 24px; border-radius: 50%; object-fit: cover;
    border: 1px solid rgba(132, 204, 22, 0.55);
  }
  .msg-agent-callsign { font-size: 11.5px; font-weight: 800; letter-spacing: 0.08em; }
  .msg-agent-tag {
    margin-left: auto; font-size: 9.5px; padding: 2px 7px; border-radius: 999px;
    background: rgba(16, 185, 129, 0.12); border: 1px solid rgba(16, 185, 129, 0.3); color: #6ee7b7;
  }
  .prompts-row {
    margin-top: 16px; display: flex; flex-wrap: wrap; justify-content: center; gap: 8px; max-width: 740px;
  }
  .stage.chat-active .prompts-row { display: none; }
  .prompt-chip {
    background: rgba(12, 23, 17, 0.78); border: 1px solid var(--border); color: #b7d2c2;
    border-radius: 999px; padding: 7px 14px; font-size: 12px; cursor: pointer; transition: all 0.15s ease;
  }
  .prompt-chip:hover { background: rgba(132, 204, 22, 0.13); border-color: rgba(132, 204, 22, 0.45); color: #ecfccb; }
  .typing { display: inline-flex; align-items: center; gap: 8px; color: #a7f3d0; font-size: 12.5px; }
  .pulse { width: 8px; height: 8px; border-radius: 50%; background: var(--lime); animation: pulseAnim 1.1s infinite ease-in-out; }
  @keyframes pulseAnim { 0%, 100% { opacity: 0.3; transform: scale(0.85); } 50% { opacity: 1; transform: scale(1.2); } }

  .hero-brand {
    display: flex; flex-direction: column; align-items: center; gap: 10px;
    margin-bottom: 24px; text-align: center;
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
  .omnibox:focus-within { border-color: var(--agent-color, #84cc16); box-shadow: 0 14px 44px rgba(0,0,0,0.55), 0 0 28px var(--agent-glow, rgba(132,204,22,0.22)); }
  .omnibox.mode-uri:focus-within { border-color: rgba(56,189,248,0.72); box-shadow: 0 14px 44px rgba(0,0,0,0.55), 0 0 28px rgba(56,189,248,0.18); }
  .omni-icon { font-size: 18px; color: var(--agent-color, #84cc16); width: 22px; text-align: center; flex-shrink: 0; }
  .omnibox.mode-uri .omni-icon { color: var(--holo); }
  .omnibox input { flex: 1; background: transparent; border: none; outline: none; color: #fff; font-size: 15.5px; padding: 9px 4px; min-width: 0; }
  .omnibox input::placeholder { color: #6f8b7b; }
  .omni-submit {
    border: none; border-radius: 999px; padding: 10px 18px; font-size: 12.5px; font-weight: 800;
    cursor: pointer; background: linear-gradient(135deg, var(--agent-color, #84cc16), #10b981); color: #031108; flex-shrink: 0;
    transition: transform 0.12s ease, opacity 0.15s ease, background 0.2s ease;
  }
  .omnibox.mode-uri .omni-submit { background: linear-gradient(135deg, #38bdf8, #0ea5e9); color: #02121c; }
  .omni-hint { margin-top: 10px; text-align: center; font-size: 11.5px; color: #6b8777; }
  .stage.chat-active .omni-hint { display: none; }

  .bottom-dock-wrap {
    width: 100%; padding: 8px 20px 14px;
    display: flex; flex-direction: column; align-items: center; gap: 10px;
    z-index: 10; margin-top: auto;
  }
  .ghost-dock {
    display: inline-flex; align-items: center; justify-content: center; flex-wrap: wrap; gap: 6px;
    padding: 8px 12px; border-radius: 20px;
    background: rgba(9, 19, 14, 0.86); border: 1px solid rgba(148, 198, 168, 0.22);
    backdrop-filter: blur(14px);
    box-shadow: 0 14px 36px rgba(0, 0, 0, 0.52), inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }
  .dock-item {
    min-width: 82px; padding: 7px 10px 6px; border-radius: 13px;
    display: flex; flex-direction: column; align-items: center; gap: 5px;
    cursor: pointer; background: transparent; border: 1px solid transparent;
    color: #cfe4d7; text-decoration: none;
    transition: transform 0.16s ease, background 0.16s ease, border-color 0.16s ease, color 0.16s ease;
  }
  .dock-item:hover {
    transform: translateY(-4px); background: rgba(132, 204, 22, 0.13);
    border-color: rgba(132, 204, 22, 0.42); color: #fff;
  }
  .dock-icon {
    width: 40px; height: 40px; border-radius: 12px; display: grid; place-items: center;
    font-size: 18px; background: rgba(14, 27, 20, 0.96); border: 1px solid rgba(148, 198, 168, 0.22);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.35); overflow: hidden;
  }
  .dock-item:hover .dock-icon { border-color: rgba(132, 204, 22, 0.58); }
  .dock-label { font-size: 10.5px; font-weight: 700; text-align: center; line-height: 1.15; }
  .crypto-footer {
    max-width: 820px; text-align: center; font-size: 10.5px; line-height: 1.45;
    color: #7ca390; padding: 4px 14px; border-radius: 999px;
    background: rgba(10, 22, 16, 0.62); border: 1px solid rgba(16, 185, 129, 0.22);
  }
  .crypto-footer strong { color: #a7f3d0; font-weight: 700; }

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
      <button type="button" class="top-btn mono" onclick="openZen()" style="border-color:rgba(56,189,248,0.5);color:#7dd3fc;" title="Read The Zen of BIFROST — Mantra & Manifesto">ᛉ ZEN</button>
      <button type="button" class="top-btn mono" id="think-toggle" onclick="toggleThinking()">🧠 THINK: OFF</button>
      <button type="button" class="top-btn mono" onclick="openQrScanner()" style="border-color:rgba(132,204,22,0.55);color:#bef264;">📷 SCAN QR</button>
      <button type="button" class="top-btn mono" id="contacts-btn" onclick="openContacts()" title="Local Sovereign Contact Book (~/.config/bifrost/contacts.json · 0 DNS)">📒 CONTACTS (0)</button>
      <button type="button" class="top-btn mono" id="nodes-btn" onclick="openDrawer()">📡 NODES (0)</button>
    </div>
  </header>

  <main class="stage" id="stage">
    <div class="hero-brand">
      <div class="brand-lockup">
        <img src="bifrost://home/icons/bifrost.png" alt="BIFROST" style="width:62px;height:62px;border-radius:16px;box-shadow:0 0 28px rgba(56,189,248,0.28);border:1px solid rgba(132,204,22,0.42);" />
        <div class="wordmark mono">BIFROST</div>
        <span class="mono" style="padding:4px 10px;border-radius:999px;font-size:11px;font-weight:800;letter-spacing:0.06em;background:rgba(56,189,248,0.14);border:1px solid rgba(56,189,248,0.5);color:#7dd3fc;">{{BIFROST_SHORT_TAG}}</span>
      </div>
      <div class="tagline mono">ZERO-IP GHOST BROWSER · POWERED BY RETICULUM · OFF-GRID REALM</div>
      <button type="button" class="new-chat-btn mono" onclick="resetChat()">✦ NEW CHAT</button>
    </div>

    <div class="council-strip" id="council-strip" role="tablist" aria-label="Ghost-Cleared Special Force">
      <button type="button" class="bot-medallion active" data-agent="heimdal" style="--m-color:#84cc16;--m-glow:rgba(132,204,22,0.28);--m-soft:rgba(132,204,22,0.16);" onclick="selectAgent('heimdal')">
        <div class="bot-orb"><img src="bifrost://home/icons/bot-heimdal.png" alt="Heimdal" /></div>
        <span class="bot-name mono">HEIMDAL</span>
        <span class="bot-role mono">Bifrost Guardian</span>
      </button>
      <button type="button" class="bot-medallion" data-agent="s3c" style="--m-color:#22c55e;--m-glow:rgba(34,197,94,0.30);--m-soft:rgba(34,197,94,0.16);" onclick="selectAgent('s3c')">
        <div class="bot-orb"><img src="bifrost://home/icons/bot-s3c.png" alt="@S3-C" /></div>
        <span class="bot-name mono">@S3-C</span>
        <span class="bot-role mono">Citadel Sentinel</span>
      </button>
      <button type="button" class="bot-medallion" data-agent="andvari" style="--m-color:#f59e0b;--m-glow:rgba(245,158,11,0.30);--m-soft:rgba(245,158,11,0.16);" onclick="selectAgent('andvari')">
        <div class="bot-orb"><img src="bifrost://home/icons/bot-andvari.png" alt="Andvari" /></div>
        <span class="bot-name mono">ANDVARI</span>
        <span class="bot-role mono">IronPass Forger</span>
      </button>
      <button type="button" class="bot-medallion" data-agent="brokk" style="--m-color:#38bdf8;--m-glow:rgba(56,189,248,0.30);--m-soft:rgba(56,189,248,0.16);" onclick="selectAgent('brokk')">
        <div class="bot-orb"><img src="bifrost://home/icons/bot-brokk.png" alt="Brokk" /></div>
        <span class="bot-name mono">BROKK</span>
        <span class="bot-role mono">Silicon &amp; RF</span>
      </button>
    </div>

    <div class="omnibox-wrap">
      <form class="omnibox" id="omnibox-form" onsubmit="handleOmnibox(event)">
        <span class="omni-icon" id="omni-icon">✦</span>
        <input
          id="omni-input"
          autocomplete="off"
          autofocus
          placeholder="Summon Heimdal · Guardian of the Bifrost — or enter bifrost://, ironpass://, @s3-c..."
          oninput="updateOmniboxMode()"
        />
        <button type="submit" class="omni-submit mono" id="omni-submit">HEIMDAL ↵</button>
      </form>
      <div class="omni-hint mono">
        Ghost-Cleared Special Force · 100% local silicon (Ventuno Q NPU / MIMIR) · Type <code>@s3-c</code>, <code>@andvari</code>, <code>@brokk</code>, or any <code>bifrost://</code> / <code>#ip=</code> URI.
      </div>
    </div>

    <div class="prompts-row" id="prompts-row"></div>

    <section class="chat-thread" id="chat-thread" aria-live="polite"></section>
  </main>

  <footer class="bottom-dock-wrap">
    <nav class="ghost-dock" id="quickbar" aria-label="Native Ghost Apps">
      <a class="dock-item" href="bifrost://home/showcase">
        <div class="dock-icon"><img src="bifrost://home/icons/citadel.png" alt="Microsite" style="width:36px;height:36px;border-radius:10px;" /></div>
        <div class="dock-label">Microsite</div>
      </a>
      <a class="dock-item" href="ironpass://local-demo/dpp/battery-01">
        <div class="dock-icon"><img src="bifrost://home/icons/ironpass.png" alt="Battery DPP" style="width:36px;height:36px;border-radius:10px;" /></div>
        <div class="dock-label">Battery DPP</div>
      </a>
      <a class="dock-item" href="ironpass://local-demo/dpp/drone-01">
        <div class="dock-icon">🛸</div>
        <div class="dock-label">UAV DPP</div>
      </a>
      <a class="dock-item" href="bifrost://home/qr-sheet">
        <div class="dock-icon">🖨️</div>
        <div class="dock-label">QR Tags</div>
      </a>
      <button type="button" class="dock-item" onclick="openDrawer()">
        <div class="dock-icon">📡</div>
        <div class="dock-label">Nodes</div>
      </button>
    </nav>
    <div class="crypto-footer mono">
      🔒 <strong>512-bit Curve25519</strong> (<strong>X25519</strong> ephemeral ECDH + <strong>Ed25519</strong> auth) · <strong>AES-256-CBC + HMAC-SHA256</strong> over Reticulum · 0 DNS · 0 TLS CAs · 0 Open Ports
    </div>
  </footer>

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
      <div class="peer-card" style="border-color:rgba(16,185,129,0.38);">
        <div style="display:flex;justify-content:space-between;align-items:center;gap:8px;">
          <div class="mono" style="font-size:11px;font-weight:800;color:#6ee7b7;">💾 ENCRYPTED ASSET CACHE (SHA-256-CTR + HMAC)</div>
          <button type="button" class="btn-sm amber mono" onclick="clearEncCache()">🧹 CLEAR</button>
        </div>
        <div id="enc-cache-info" class="mono" style="font-size:11px;color:var(--muted);">0 files · 0.0 MB (static JS/CSS/3D models only · dynamic routes bypassed)</div>
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

  <div class="drawer-backdrop" id="zen-backdrop" onclick="if(event.target===this) closeZen()">
    <aside class="drawer">
      <div class="drawer-head">
        <h2 class="mono" style="font-size:15px;">ᛉ THE ZEN OF BIFROST</h2>
        <button type="button" class="top-btn mono" onclick="closeZen()">✕ CLOSE</button>
      </div>
      <div class="peer-card" style="border-color:rgba(132,204,22,0.5); background:linear-gradient(150deg, rgba(16,185,129,0.12), rgba(56,189,248,0.08));">
        <div class="mono" style="font-size:10.5px; font-weight:800; color:var(--lime); letter-spacing:0.1em;">THE MANTRA · NORTH STAR</div>
        <div style="font-size:14px; font-weight:800; color:#fff; line-height:1.45;">“What cannot be seen cannot be attacked.”</div>
        <div style="font-size:12px; color:#d4ebe0; line-height:1.6;">
          <strong>BIFROST</strong> is the 100% sovereign, zero-IP Ghost Realm — a new way of computing and running local agentic AI with <strong>no cloud, no DNS, no public IP addresses, and zero open server ports</strong>. Powered by heavy end-to-end encryption over <strong>Reticulum</strong>, BIFROST complements <strong>ASGAARD</strong> wherever absolute discretion, invisibility, and off-grid resilience matter most — starting on a palm-sized <strong>Ventuno Q (40 TOPS NPU)</strong> and scaling seamlessly to <strong>MIMIR</strong> local super-LLM servers (<strong>32 GB VRAM NVIDIA Ubuntu Linux</strong>, <strong>128 GB NVIDIA DGX Spark</strong> enclaves, and beyond) — on hardware you own.
        </div>
      </div>
      <div class="peer-card" style="border-color:rgba(56,189,248,0.35); gap:10px;">
        <div class="mono" style="font-size:10.5px; font-weight:800; color:#7dd3fc; letter-spacing:0.08em;">TEN PRINCIPLES OF THE GHOST REALM</div>
        <ol style="padding-left:18px; display:flex; flex-direction:column; gap:8px; font-size:12px; color:#d4e8dc; line-height:1.55;">
          <li><strong>Invisibility is the highest form of defense.</strong> Reachable means attackable. Eliminating public IPs, DNS, TLS CAs, and open ports removes the target before a scanner can knock.</li>
          <li><strong>Cross the bridge; leave the old world behind.</strong> Zero cookies, zero trackers, zero cloud decryption. End-to-end <strong>512-bit Curve25519 (X25519 + Ed25519)</strong>, <strong>AES-256-CBC + HMAC-SHA256</strong>, IFAC masking, and mutual Ed25519 hardware auth.</li>
          <li><strong>Zero trust in the wire — total freedom of transport.</strong> Fiber, Starlink, 5G, and Wi-Fi are untrusted pipes; out-of-band <strong>Sub-GHz LoRa</strong> carries emergency C2, telemetry, and offline DPPs when broadband goes dark.</li>
          <li><strong>Lean beats bloated. Every kilobyte is sacred.</strong> BIFROST is not a second legacy OS. Native Reticulum Ghost apps stay ultra-lean, zero-bloat, and fast across any link.</li>
          <li><strong>Your silicon, your mind.</strong> 100% local AI — starting on <strong>Ventuno Q (40 TOPS NPU)</strong> and scaling cleanly to <strong>MIMIR</strong> super-LLM servers (<strong>32 GB NVIDIA Ubuntu</strong> &amp; <strong>128 GB NVIDIA DGX Spark</strong>).</li>
          <li><strong>Two realms, one purpose — never a duplicate OS.</strong> <strong>ASGAARD</strong> powers broad enterprise/GPU-farm studios; <strong>BIFROST</strong> is the lean, off-grid Ghost Realm; <strong>CITADEL</strong> is the 0-port Linux fabric; <strong>IRONPASS</strong> is physical/cryptographic proof.</li>
          <li><strong>Identity belongs to hardware, not passwords.</strong> Persistent <strong>Ed25519 device keys</strong> and physical <strong>NTAG 424 DNA / Hybrid #ip=</strong> tags replace fragile Web2 logins.</li>
          <li><strong>Open wire, clean-room craft, honest engineering.</strong> Clean-room MIT Rust &amp; C++ over the public-domain Reticulum Wire Protocol (credit to inventor <strong>Mark Qvist</strong>) — strictly defensive and sovereign.</li>
          <li><strong>Ephemeral on the client, sovereign in the enclave.</strong> RAM-only incognito workstations + encrypted static cache; persistent truth stays inside your 0-port Linux enclaves.</li>
          <li><strong>Guided by curiosity, imagination, and craft.</strong> Exploring what computing becomes when privacy, autonomy, and local AI are the default laws of physics.</li>
        </ol>
      </div>
      <div class="peer-card" style="border-color:rgba(245,158,11,0.38);">
        <div class="mono" style="font-size:10.5px; font-weight:800; color:#fcd34d; letter-spacing:0.08em;">THE 4-QUESTION DECISION FILTER</div>
        <div class="mono" style="font-size:11px; color:#d4e8dc; line-height:1.6;">
          1. Works with 0 internet, 0 DNS, 0 cloud APIs?<br/>
          2. Lean &amp; native to Reticulum (400 B packets, zero Web2 bloat)?<br/>
          3. Preserves invisibility &amp; zero-IP discretion (0 open ports)?<br/>
          4. Scales across local silicon (Ventuno Q → MIMIR 32 GB GPU → MIMIR 128 GB DGX Spark)?
        </div>
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
let activeAgent = 'heimdal';

const GHOST_AGENTS = {
  heimdal: {
    id: 'heimdal',
    callsign: 'HEIMDAL',
    role: 'Bifrost Guardian',
    icon: 'bifrost://home/icons/bot-heimdal.png',
    color: '#84cc16',
    glow: 'rgba(132, 204, 22, 0.28)',
    soft: 'rgba(132, 204, 22, 0.16)',
    placeholder: 'Summon Heimdal · Guardian of the Bifrost — or enter bifrost://, ironpass://, @s3-c...',
    prompts: [
      ['Why 0-port BIFROST for CIRPASS-2?', 'Why does BIFROST with 0 open ports beat a regular HTTPS browser for CIRPASS-2 Digital Product Passports?'],
      ['The Zen of BIFROST in 3 lines', 'Summarize The Zen of BIFROST and how it complements Asgaard without duplicating an OS.'],
      ['Ventuno Q to MIMIR scaling', 'How does local sovereign AI inside BIFROST scale from a Ventuno Q 40 TOPS NPU to MIMIR 32GB/128GB Linux nodes?'],
    ],
  },
  s3c: {
    id: 's3c',
    callsign: '@S3-C',
    role: 'Citadel Sentinel',
    icon: 'bifrost://home/icons/bot-s3c.png',
    color: '#22c55e',
    glow: 'rgba(34, 197, 94, 0.30)',
    soft: 'rgba(34, 197, 94, 0.16)',
    placeholder: 'Summon @S3-C · Citadel Zero-IP Security Sentinel (0 open ports, IFAC, Lock 2)...',
    prompts: [
      ['Explain 0-port [CITADEL-DROP]', 'Explain how nftables [CITADEL-DROP] with outbound Reticulum links eliminates public attack surfaces.'],
      ['Lock 1 IFAC vs Lock 2 Ed25519', 'Compare Lock 1 IFAC dark-network masking and Lock 2 mutual Ed25519 client allowlists in Citadel.'],
      ['Zero-IP failover over LoRa', 'How does Citadel maintain C2 and kill-switches over 868/915 MHz LoRa when Fiber and 5G go dark?'],
    ],
  },
  andvari: {
    id: 'andvari',
    callsign: 'ANDVARI',
    role: 'IronPass Forger',
    icon: 'bifrost://home/icons/bot-andvari.png',
    color: '#f59e0b',
    glow: 'rgba(245, 158, 11, 0.30)',
    soft: 'rgba(245, 158, 11, 0.16)',
    placeholder: 'Summon Andvari · IronPass EU ESPR, Hybrid #ip= & GS1 EPCIS 2.0 Ledger Forger...',
    prompts: [
      ['How Hybrid #ip= tags split tiers', 'Explain how a Hybrid IronPass tag splits public EU ESPR consumer data from the classified IronPass Ledger over Reticulum.'],
      ['NTAG 424 DNA anti-cloning', 'How does NXP NTAG 424 DNA AES-128 CMAC SUN verification prevent physical product tag cloning offline?'],
      ['Hash-chained GS1 EPCIS 2.0', 'Why is a hash-chained, Ed25519-signed GS1 EPCIS 2.0 IronPass Ledger tamper-evident without a public blockchain?'],
    ],
  },
  brokk: {
    id: 'brokk',
    callsign: 'BROKK',
    role: 'Silicon & RF',
    icon: 'bifrost://home/icons/bot-brokk.png',
    color: '#38bdf8',
    glow: 'rgba(56, 189, 248, 0.30)',
    soft: 'rgba(56, 189, 248, 0.16)',
    placeholder: 'Summon Brokk · Ventuno Q NPU, MIMIR GPU enclaves & Sub-GHz LoRa hardware...',
    prompts: [
      ['Multi-Ventuno scaling blueprint', 'How do we scale multiple Ventuno Q 40 TOPS NPU boxes inside one zero-IP BIFROST enclave?'],
      ['LoRa vs broadband RNS links', 'Explain the bandwidth boundary between 868 MHz LoRa (~1.7 kbps SF9) and Starlink/Fiber Reticulum links.'],
      ['Hardware crypto on ARMv8 & x86', 'How do ARMv8 AES/SHA2 extensions on Ventuno Q and AES-NI on x86 accelerate clean-room Rust Reticulum?'],
    ],
  },
};

function renderStarterPrompts() {
  const row = document.getElementById('prompts-row');
  if (!row) return;
  const ag = GHOST_AGENTS[activeAgent] || GHOST_AGENTS.heimdal;
  row.innerHTML = ag.prompts.map(([label, prompt]) =>
    `<button type="button" class="prompt-chip" onclick="askPreset(${JSON.stringify(prompt).replace(/"/g, '&quot;')})">${escHtml(label)}</button>`
  ).join('');
}

function selectAgent(id, focusInput = true) {
  if (!GHOST_AGENTS[id]) return;
  activeAgent = id;
  const ag = GHOST_AGENTS[id];
  const stage = document.getElementById('stage');
  if (stage) {
    stage.style.setProperty('--agent-color', ag.color);
    stage.style.setProperty('--agent-glow', ag.glow);
    stage.style.setProperty('--agent-soft', ag.soft);
  }
  document.querySelectorAll('.bot-medallion').forEach(b => {
    b.classList.toggle('active', b.getAttribute('data-agent') === id);
  });
  const inp = document.getElementById('omni-input');
  if (inp) {
    inp.placeholder = ag.placeholder;
    if (focusInput) inp.focus();
  }
  renderStarterPrompts();
  updateOmniboxMode();
}

function detectMentionAgent(raw) {
  const v = (raw || '').trim().toLowerCase();
  if (v.startsWith('@s3-c') || v.startsWith('@s3c')) return 's3c';
  if (v.startsWith('@andvari')) return 'andvari';
  if (v.startsWith('@brokk')) return 'brokk';
  if (v.startsWith('@heimdal')) return 'heimdal';
  return null;
}

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
  const mentioned = detectMentionAgent(input.value);
  if (mentioned && mentioned !== activeAgent) {
    selectAgent(mentioned, false);
  }
  const ag = GHOST_AGENTS[activeAgent] || GHOST_AGENTS.heimdal;
  const detected = detectGhostUri(input.value);
  const isCard = Boolean(detected && detected.startsWith('bifrost-card://'));
  const isUri = Boolean(detected);
  form.classList.toggle('mode-uri', isUri);
  icon.textContent = isCard ? '📒' : (isUri ? '🔒' : '✦');
  if (!isBusy) btn.textContent = isCard ? 'IMPORT CARD →' : (isUri ? 'OPEN URI →' : `${ag.callsign} ↵`);
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
  if (role === 'assistant') {
    const ag = GHOST_AGENTS[(meta && meta.agent) || activeAgent] || GHOST_AGENTS.heimdal;
    bubble.style.borderColor = ag.glow;
    bubble.innerHTML += `
      <div class="msg-agent-head">
        <img src="${ag.icon}" alt="${escHtml(ag.callsign)}" style="border-color:${ag.color};" />
        <span class="msg-agent-callsign mono" style="color:${ag.color};">${escHtml(ag.callsign)}</span>
        <span class="mono" style="font-size:10px;color:var(--muted);">· ${escHtml(ag.role)}</span>
        <span class="msg-agent-tag mono">GHOST-CLEARED · LOCAL SILICON</span>
      </div>`;
  }
  bubble.innerHTML += escHtml(content).replace(/\n/g, '<br/>');
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
  const inp = document.getElementById('omni-input');
  inp.value = '';
  updateOmniboxMode();
  inp.focus();
}

function askPreset(text) {
  const input = document.getElementById('omni-input');
  input.value = text;
  updateOmniboxMode();
  handleOmnibox(new Event('submit'));
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
  if (uri) { if (window.__bifrostShowTransit) window.__bifrostShowTransit(uri); window.location.href = uri; return; }

  input.value = '';
  updateOmniboxMode();
  appendMessage('user', raw);
  conversation.push({ role: 'user', content: raw });
  isBusy = true;
  const submitBtn = document.getElementById('omni-submit');
  submitBtn.disabled = true;
  submitBtn.textContent = 'AI...';

  const ag = GHOST_AGENTS[activeAgent] || GHOST_AGENTS.heimdal;
  const thread = document.getElementById('chat-thread');
  const pending = document.createElement('div');
  pending.className = 'msg assistant';
  pending.innerHTML = `<div class="bubble" style="border-color:${ag.glow};"><span class="typing mono"><img src="${ag.icon}" style="width:20px;height:20px;border-radius:50%;border:1px solid ${ag.color};" /><span class="pulse" style="background:${ag.color};"></span>Summoning ${escHtml(ag.callsign)} over encrypted Reticulum → Local Silicon (${escHtml(llmModel)})...</span></div>`;
  thread.appendChild(pending);
  window.scrollTo({ top: document.body.scrollHeight, behavior: 'smooth' });

  try {
    const bodyStr = JSON.stringify({ agent: activeAgent, model: llmModel, enable_thinking: enableThinking, messages: conversation.slice(-12) });
    const encoded = encodeURIComponent(bodyStr);
    const r = encoded.length <= 6000
      ? await fetch(`/api/ai-chat?payload=${encoded}&_t=${Date.now()}`)
      : await fetch('/api/ai-chat', {
          method: 'POST',
          headers: { 'Content-Type': 'text/plain;charset=UTF-8' },
          body: bodyStr,
        });
    const d = await r.json();
    pending.remove();
    if (!d.ok) appendMessage('error', d.error || 'Local AI peer unavailable.');
    else { conversation.push({ role: 'assistant', content: d.reply || '' }); appendMessage('assistant', d.reply || '', d); }
  } catch (err) {
    pending.remove();
    appendMessage('error', `Reticulum transport error: ${err}`);
  } finally {
    isBusy = false;
    submitBtn.disabled = false;
    updateOmniboxMode();
  }
}

function openZen() { document.getElementById('zen-backdrop').classList.add('open'); }
function closeZen() { document.getElementById('zen-backdrop').classList.remove('open'); }
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

async function clearEncCache() {
  const info = document.getElementById('enc-cache-info');
  if (info) info.textContent = 'Clearing encrypted asset cache...';
  try {
    const r = await fetch(`/api/cache/clear?_t=${Date.now()}`);
    const d = await r.json();
    if (info) info.textContent = `Cleared ${d.removed || 0} files · 0.0 MB encrypted on disk`;
    refreshPeers();
  } catch (_) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(JSON.stringify({ cmd: 'clear_cache' }));
      if (info) info.textContent = 'Cleared via native IPC.';
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
    const cacheEl = document.getElementById('enc-cache-info');
    if (cacheEl && typeof d.cache_files === 'number') {
      const mb = ((d.cache_bytes || 0) / (1024 * 1024)).toFixed(2);
      cacheEl.textContent = `${d.cache_files} encrypted files · ${mb} MB (static JS/CSS/3D twins · <15 ms reload)`;
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

    // Dynamic bottom Ghost Dock from ~/.config/bifrost/bookmarks.json or discovered peers
    const qb = document.getElementById('quickbar');
    if (qb) {
      if (bookmarks.length) {
        qb.innerHTML = bookmarks.map(b => `<a class="dock-item" href="${escHtml(b.uri)}"><div class="dock-icon">${escHtml(b.icon||'🔒')}</div><div class="dock-label">${escHtml(b.label)}</div></a>`).join('')
          + `<button type="button" class="dock-item" onclick="openDrawer()"><div class="dock-icon">📡</div><div class="dock-label">Nodes</div></button>`;
      } else if (peers.length) {
        qb.innerHTML = peers.slice(0, 6).map(p => `<a class="dock-item" href="bifrost://${p.hash}/"><div class="dock-icon">🔒</div><div class="dock-label">${escHtml(p.alias.slice(0, 12))}</div></a>`).join('')
          + `<a class="dock-item" href="bifrost://home/qr-sheet"><div class="dock-icon">🖨️</div><div class="dock-label">QR Tags</div></a>`
          + `<button type="button" class="dock-item" onclick="openDrawer()"><div class="dock-icon">📡</div><div class="dock-label">Nodes</div></button>`;
      }
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

selectAgent('heimdal', false);
refreshContacts();
refreshPeers();
setInterval(refreshPeers, 3000);
</script>
</body>
</html>
"##;

pub const HUD_INIT_SCRIPT: &str = r##"
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
  window.__bifrostHudMounted = true;

  const isHomePage = (window.location.protocol === 'bifrost:' || window.location.protocol === 'citadel:') && window.location.hostname === 'home';

  // === BIFROST Animated Reticulum Encryption Transit & Live Asset Stream Overlay ===
  const activeStreams = new Map();
  let streamSeq = 0;
  let navTransitActive = false;
  let overlayVisible = false;
  let overlayMode = 'modal';
  let overlayTarget = window.location.href;
  let overlayStart = Date.now();
  let showTimer = null;
  let hideTimer = null;
  let tickerTimer = null;
  let userDismissed = false;
  let pageBootDone = document.readyState === 'complete';
  let cacheToastTimer = null;

  function shortTargetLabel(raw) {
    const s = String(raw || '').trim();
    if (s.length > 52) return s.slice(0, 26) + '…' + s.slice(-18);
    return s || 'Reticulum Ghost Vault';
  }

  function cleanAssetName(url) {
    try {
      const u = String(url || '').split('?')[0].split('#')[0];
      const parts = u.split('/').filter(Boolean);
      return parts.length ? parts[parts.length - 1] : u;
    } catch (_) {
      return 'encrypted-stream';
    }
  }

  function shouldTrackUrl(url) {
    const s = String(url || '').toLowerCase();
    if (!s || s.startsWith('data:') || s.startsWith('blob:')) return false;
    // Never pop the streaming dock for background API/cluster/health/camera polling
    if (
      s.includes('/api/') ||
      s.includes('/__bifrost_rns/') ||
      s.includes('/cluster') ||
      s.includes('/health') ||
      s.includes('/status') ||
      s.includes('/events') ||
      s.includes('/cam/')
    ) {
      return false;
    }
    const clean = s.split('?')[0].split('#')[0];
    if (clean.includes('/models/') || clean.includes('/assets/') || clean.includes('/_next/static/')) {
      return true;
    }
    return /\.(glb|gltf|obj|stl|ply|wasm|js|mjs|css|png|jpg|jpeg|webp|mp4|pdf)$/i.test(clean);
  }

  function injectLoaderStyles() {
    if (document.getElementById('__bf_rns_loader_css')) return;
    const st = document.createElement('style');
    st.id = '__bf_rns_loader_css';
    st.textContent = `
      @keyframes __bfSpin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
      @keyframes __bfPulse { 0%,100% { opacity: .45; transform: scale(.96); } 50% { opacity: 1; transform: scale(1.05); } }
      @keyframes __bfFlow { 0% { stroke-dashoffset: 64; } 100% { stroke-dashoffset: 0; } }
      @keyframes __bfShimmer { 0% { transform: translateX(-100%); } 100% { transform: translateX(260%); } }
      @keyframes __bfFadeIn { from { opacity: 0; transform: translateY(8px) scale(.98); } to { opacity: 1; transform: translateY(0) scale(1); } }
      #__bf_rns_overlay {
        position: fixed; z-index: 2147483646; pointer-events: none;
        font-family: ui-monospace, 'JetBrains Mono', SFMono-Regular, Menlo, monospace;
        color: #ecfdf5; transition: opacity .28s ease, transform .28s ease;
        opacity: 0; display: none;
      }
      #__bf_rns_overlay.visible { display: flex; opacity: 1; }
      #__bf_rns_overlay.mode-modal {
        inset: 34px 0 0 0; align-items: center; justify-content: center;
        background: radial-gradient(circle at 50% 40%, rgba(4, 24, 16, 0.86), rgba(2, 6, 4, 0.92));
        backdrop-filter: blur(8px); pointer-events: auto;
      }
      #__bf_rns_overlay.mode-dock { right: 16px; bottom: 16px; width: min(410px, calc(100vw - 32px)); pointer-events: auto; }
      .__bf_card {
        width: 100%; max-width: 500px;
        background: linear-gradient(160deg, rgba(5, 16, 11, 0.97), rgba(3, 10, 14, 0.97));
        border: 1px solid rgba(56, 189, 248, 0.45); border-radius: 14px; padding: 16px 18px;
        box-shadow: 0 18px 54px rgba(0, 0, 0, 0.72), 0 0 28px rgba(16, 185, 129, 0.16);
        animation: __bfFadeIn .22s ease-out;
      }
      #__bf_rns_overlay.mode-dock .__bf_card { padding: 13px 15px; border-color: rgba(132, 204, 22, 0.48); }
      .__bf_head { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 10px; }
      .__bf_title { font-size: 11.5px; font-weight: 800; letter-spacing: .08em; color: #bef264; display: flex; align-items: center; gap: 7px; }
      .__bf_dot { width: 8px; height: 8px; border-radius: 50%; background: #38bdf8; box-shadow: 0 0 10px #38bdf8; animation: __bfPulse 1.1s infinite ease-in-out; }
      .__bf_meta { display: flex; align-items: center; gap: 8px; font-size: 10.5px; color: #7dd3fc; }
      .__bf_close { background: rgba(148,163,184,0.12); border: 1px solid rgba(148,163,184,0.28); color: #cbd5e1; border-radius: 6px; padding: 2px 7px; font-size: 10px; cursor: pointer; font-family: inherit; }
      .__bf_close:hover { background: rgba(248,113,113,0.2); color: #fecaca; border-color: rgba(248,113,113,0.45); }
      .__bf_target { font-size: 11px; color: #e0f2fe; background: rgba(2, 8, 5, 0.85); border: 1px solid rgba(56, 189, 248, 0.28); border-radius: 8px; padding: 5px 9px; margin-bottom: 11px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
      .__bf_bridge { width: 100%; height: 68px; display: block; margin-bottom: 10px; }
      .__bf_steps { display: flex; flex-direction: column; gap: 5px; margin-bottom: 10px; font-size: 10.5px; }
      .__bf_step { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 4px 8px; border-radius: 7px; background: rgba(2, 9, 6, 0.72); border: 1px solid rgba(148, 198, 168, 0.14); }
      .__bf_step.done { color: #a7f3d0; border-color: rgba(16, 185, 129, 0.30); }
      .__bf_step.active { color: #e0f2fe; border-color: rgba(56, 189, 248, 0.45); background: rgba(8, 30, 42, 0.48); }
      .__bf_bar_wrap { height: 5px; border-radius: 999px; background: rgba(15, 23, 42, 0.9); overflow: hidden; border: 1px solid rgba(56,189,248,0.25); margin-bottom: 8px; }
      .__bf_bar_fill { width: 42%; height: 100%; border-radius: 999px; background: linear-gradient(90deg, #10b981, #38bdf8, #84cc16); animation: __bfShimmer 1.35s infinite linear; }
      .__bf_cipher { font-size: 9.5px; color: #6ee7b7; opacity: .82; display: flex; justify-content: space-between; gap: 8px; overflow: hidden; white-space: nowrap; }
      #__bf_cache_toast { position: fixed; top: 42px; right: 14px; z-index: 2147483646; padding: 5px 11px; border-radius: 999px; background: rgba(4, 24, 14, 0.94); border: 1px solid rgba(132, 204, 22, 0.55); color: #bef264; font-family: ui-monospace, Menlo, monospace; font-size: 10.5px; font-weight: 800; box-shadow: 0 6px 20px rgba(0,0,0,0.5); pointer-events: none; opacity: 0; transition: opacity .22s ease; }
      #__bf_cache_toast.show { opacity: 1; }
    `;
    (document.head || document.documentElement).appendChild(st);
  }

  function ensureOverlayEl() {
    if (!document.body) return null;
    injectLoaderStyles();
    let el = document.getElementById('__bf_rns_overlay');
    if (el) return el;
    el = document.createElement('div');
    el.id = '__bf_rns_overlay';
    el.innerHTML = `
      <div class="__bf_card">
        <div class="__bf_head">
          <div class="__bf_title"><span class="__bf_dot"></span><span>ᛉ BIFROST // RETICULUM ENCRYPTED TUNNEL</span></div>
          <div class="__bf_meta">
            <span id="__bf_rns_elapsed">0.0s</span>
            <button type="button" class="__bf_close" id="__bf_rns_hide">✕</button>
          </div>
        </div>
        <div class="__bf_target" id="__bf_rns_target">Resolving 128-bit Reticulum destination…</div>
        <svg class="__bf_bridge" viewBox="0 0 460 68" fill="none" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <linearGradient id="__bfGrad" x1="0" y1="0" x2="460" y2="0" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="#84cc16"/>
              <stop offset="50%" stop-color="#38bdf8"/>
              <stop offset="100%" stop-color="#10b981"/>
            </linearGradient>
          </defs>
          <path d="M 88 34 Q 160 12 230 34 T 372 34" stroke="rgba(56,189,248,0.22)" stroke-width="3" fill="none"/>
          <path d="M 88 34 Q 160 12 230 34 T 372 34" stroke="url(#__bfGrad)" stroke-width="2.5" stroke-dasharray="10 6" style="animation:__bfFlow 0.85s linear infinite;" fill="none"/>
          <path d="M 88 34 Q 160 56 230 34 T 372 34" stroke="rgba(132,204,22,0.22)" stroke-width="2" stroke-dasharray="6 10" style="animation:__bfFlow 1.25s linear infinite reverse;" fill="none"/>
          <circle r="4" fill="#bef264"><animateMotion dur="1.15s" repeatCount="indefinite" path="M 88 34 Q 160 12 230 34 T 372 34"/></circle>
          <circle r="3.5" fill="#38bdf8"><animateMotion dur="1.15s" begin="0.38s" repeatCount="indefinite" path="M 88 34 Q 160 12 230 34 T 372 34"/></circle>
          <circle r="3" fill="#34d399"><animateMotion dur="1.45s" begin="0.2s" repeatCount="indefinite" path="M 372 34 Q 300 56 230 34 T 88 34"/></circle>
          <rect x="6" y="14" width="84" height="40" rx="9" fill="#06140d" stroke="#84cc16" stroke-width="1.3"/>
          <text x="48" y="31" text-anchor="middle" fill="#bef264" font-size="8.5" font-weight="800" font-family="ui-monospace,monospace">WORKSTATION</text>
          <text x="48" y="44" text-anchor="middle" fill="#7ea18d" font-size="7.5" font-family="ui-monospace,monospace">0 HTTP PORTS</text>
          <g transform="translate(230,34)">
            <circle r="21" fill="#04151e" stroke="#38bdf8" stroke-width="1.3"/>
            <circle r="26" fill="none" stroke="#84cc16" stroke-width="1" stroke-dasharray="5 5" style="transform-origin:0 0;animation:__bfSpin 5s linear infinite;"/>
            <text x="0" y="-2" text-anchor="middle" fill="#7dd3fc" font-size="8" font-weight="800" font-family="ui-monospace,monospace">X25519</text>
            <text x="0" y="9" text-anchor="middle" fill="#bef264" font-size="7.5" font-weight="800" font-family="ui-monospace,monospace">AES-256</text>
          </g>
          <rect x="370" y="14" width="84" height="40" rx="9" fill="#06140d" stroke="#10b981" stroke-width="1.3"/>
          <text x="412" y="31" text-anchor="middle" fill="#6ee7b7" font-size="8.5" font-weight="800" font-family="ui-monospace,monospace">GHOST VAULT</text>
          <text x="412" y="44" text-anchor="middle" fill="#7ea18d" font-size="7.5" font-family="ui-monospace,monospace">0 OPEN PORTS</text>
        </svg>
        <div class="__bf_steps">
          <div class="__bf_step done"><span>✓ 01 · ZERO-DNS 128-BIT HASH</span><span>VERIFIED</span></div>
          <div class="__bf_step done"><span>✓ 02 · 512-BIT CURVE25519 LINK</span><span>X25519 + Ed25519</span></div>
          <div class="__bf_step active"><span id="__bf_rns_step3">⚡ 03 · STREAMING ENCRYPTED CHT2 CHUNKS…</span><span id="__bf_rns_badge">LIVE</span></div>
        </div>
        <div class="__bf_bar_wrap"><div class="__bf_bar_fill"></div></div>
        <div class="__bf_cipher">
          <span id="__bf_rns_hex">CHT2 AES-256-CBC · HMAC-SHA256</span>
          <span>0 DNS · 0 TLS CAs</span>
        </div>
      </div>
    `;
    document.body.appendChild(el);
    const hideBtn = document.getElementById('__bf_rns_hide');
    if (hideBtn) hideBtn.onclick = () => { userDismissed = true; hideOverlayImmediate(); };
    return el;
  }

  function flashCacheToast(label) {
    if (!document.body) return;
    injectLoaderStyles();
    let t = document.getElementById('__bf_cache_toast');
    if (!t) { t = document.createElement('div'); t.id = '__bf_cache_toast'; document.body.appendChild(t); }
    t.textContent = `💾 ENCRYPTED CACHE HIT · ${label}`;
    t.classList.add('show');
    if (cacheToastTimer) clearTimeout(cacheToastTimer);
    cacheToastTimer = setTimeout(() => { t.classList.remove('show'); }, 1500);
  }

  function randCipherHex() {
    const h = '0123456789abcdef';
    let out = '0x';
    for (let i = 0; i < 16; i++) out += h[(Math.random() * 16) | 0];
    return out;
  }

  function renderOverlayContent() {
    const el = ensureOverlayEl();
    if (!el) return;
    const elapsed = ((Date.now() - overlayStart) / 1000).toFixed(1);
    const elEl = document.getElementById('__bf_rns_elapsed');
    const tgtEl = document.getElementById('__bf_rns_target');
    const step3 = document.getElementById('__bf_rns_step3');
    const badge = document.getElementById('__bf_rns_badge');
    const hexEl = document.getElementById('__bf_rns_hex');
    if (elEl) elEl.textContent = `${elapsed}s`;
    if (tgtEl) tgtEl.textContent = shortTargetLabel(overlayTarget);
    const items = Array.from(activeStreams.values());
    if (navTransitActive) {
      if (step3) step3.textContent = '⚡ 03 · NEGOTIATING ENCRYPTED RETICULUM PAYLOAD…';
      if (badge) badge.textContent = 'TRANSIT';
    } else if (items.length > 0) {
      const extra = items.length > 1 ? ` (+${items.length - 1} queued)` : '';
      if (step3) step3.textContent = `⚡ 03 · STREAMING ${items[0].label}${extra}`;
      if (badge) badge.textContent = `${items.length} ACTIVE`;
    } else if (!pageBootDone) {
      if (step3) step3.textContent = '⚡ 03 · DECRYPTING SOVEREIGN APP BUNDLE…';
      if (badge) badge.textContent = 'LOADING';
    }
    if (hexEl) hexEl.textContent = `CHT2 ${randCipherHex()} · AES-256-CBC + HMAC`;
  }

  function showOverlay(mode, target) {
    if (userDismissed && !navTransitActive) return;
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
    overlayMode = mode;
    if (target) overlayTarget = target;
    if (!overlayVisible) overlayStart = Date.now();
    const el = ensureOverlayEl();
    if (!el) return;
    el.className = `visible mode-${overlayMode}`;
    overlayVisible = true;
    renderOverlayContent();
    if (!tickerTimer) tickerTimer = setInterval(renderOverlayContent, 160);
  }

  function hideOverlayImmediate() {
    if (showTimer) { clearTimeout(showTimer); showTimer = null; }
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
    if (tickerTimer) { clearInterval(tickerTimer); tickerTimer = null; }
    overlayVisible = false;
    const el = document.getElementById('__bf_rns_overlay');
    if (el) el.className = '';
  }

  function scheduleOverlayUpdate() {
    if (navTransitActive) { showOverlay('modal', overlayTarget); return; }
    const busy = activeStreams.size > 0 || (!isHomePage && !pageBootDone);
    if (busy) {
      if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
      if (!overlayVisible && !showTimer && !userDismissed) {
        const mode = pageBootDone ? 'dock' : 'modal';
        showTimer = setTimeout(() => {
          showTimer = null;
          if (activeStreams.size > 0 || (!isHomePage && !pageBootDone)) showOverlay(mode, window.location.href);
        }, 260);
      } else if (overlayVisible) {
        renderOverlayContent();
      }
    } else {
      if (showTimer) { clearTimeout(showTimer); showTimer = null; }
      if (overlayVisible) {
        const step3 = document.getElementById('__bf_rns_step3');
        const badge = document.getElementById('__bf_rns_badge');
        if (step3) step3.textContent = '✓ 03 · DECRYPTED & VERIFIED IN LOCAL RAM';
        if (badge) badge.textContent = 'DONE';
        if (hideTimer) clearTimeout(hideTimer);
        hideTimer = setTimeout(() => { userDismissed = false; hideOverlayImmediate(); }, 380);
      }
    }
  }

  function beginTransit(url) {
    if (!url || !/^(bifrost|citadel|ironpass):\/\//i.test(url)) return;
    if (/^(bifrost|citadel):\/\/home\/?$/i.test(url)) return;
    userDismissed = false;
    navTransitActive = true;
    overlayTarget = url;
    overlayStart = Date.now();
    showOverlay('modal', url);
  }
  window.__bifrostShowTransit = beginTransit;

  if (typeof window.fetch === 'function') {
    const origFetch = window.fetch.bind(window);
    window.fetch = function(input, init) {
      const url = typeof input === 'string' ? input : (input && input.url) || '';
      if (!shouldTrackUrl(url)) return origFetch(input, init);
      const id = ++streamSeq;
      const label = cleanAssetName(url);
      activeStreams.set(id, { label, startedAt: Date.now() });
      scheduleOverlayUpdate();
      return origFetch(input, init).then((resp) => {
        try {
          if (resp && resp.headers && resp.headers.get('x-bifrost-cache') === 'HIT-ENC') {
            if (/\.(glb|gltf|obj|stl|js|wasm)$/i.test(label)) flashCacheToast(label);
          }
        } catch (_) {}
        activeStreams.delete(id);
        scheduleOverlayUpdate();
        return resp;
      }, (err) => {
        activeStreams.delete(id);
        scheduleOverlayUpdate();
        throw err;
      });
    };
  }

  if (typeof window.XMLHttpRequest === 'function') {
    const origOpen = XMLHttpRequest.prototype.open;
    const origSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.open = function(method, url) {
      this.__bfUrl = typeof url === 'string' ? url : String(url || '');
      return origOpen.apply(this, arguments);
    };
    XMLHttpRequest.prototype.send = function() {
      const url = this.__bfUrl || '';
      if (shouldTrackUrl(url)) {
        const id = ++streamSeq;
        const label = cleanAssetName(url);
        activeStreams.set(id, { label, startedAt: Date.now() });
        scheduleOverlayUpdate();
        this.addEventListener('loadend', () => {
          try {
            if (this.getResponseHeader && this.getResponseHeader('x-bifrost-cache') === 'HIT-ENC') {
              if (/\.(glb|gltf|obj|stl|js|wasm)$/i.test(label)) flashCacheToast(label);
            }
          } catch (_) {}
          activeStreams.delete(id);
          scheduleOverlayUpdate();
        }, { once: true });
      }
      return origSend.apply(this, arguments);
    };
  }

  document.addEventListener('click', (ev) => {
    const a = ev.target && ev.target.closest ? ev.target.closest('a[href]') : null;
    if (!a) return;
    const href = a.getAttribute('href') || '';
    if (/^(bifrost|citadel|ironpass):\/\//i.test(href)) beginTransit(href);
  }, true);

  window.addEventListener('pageshow', () => {
    navTransitActive = false;
    if (document.readyState === 'complete') { pageBootDone = true; scheduleOverlayUpdate(); }
  });
  window.addEventListener('load', () => { pageBootDone = true; navTransitActive = false; scheduleOverlayUpdate(); });

  if (!isHomePage && !pageBootDone) scheduleOverlayUpdate();
  if (isHomePage) return;

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
        if (v) { beginTransit(v); window.location.href = v; }
      }
    });
    document.getElementById('__bf_home').onclick = () => { window.location.href = 'bifrost://home/'; };
    document.getElementById('__bf_back').onclick = () => window.history.back();
    document.getElementById('__bf_reload').onclick = () => { beginTransit(window.location.href); window.location.reload(); };
  }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', mountHud);
  else mountHud();
})();
"##;
