//! BIFROST — Built-in Sovereign Start Page (Google-Style Omnibox + Local AI) & Floating HUD.
//! Served strictly from Rust memory on `bifrost://home/` (0 network, 0 HTTP sockets).
//! Zero hardcoded private hashes: dynamically discovers connected Reticulum peers and optionally
//! loads operator bookmarks from `~/.config/bifrost/bookmarks.json`.

pub const HOME_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>BIFROST — The Reticulum Ghost Browser</title>
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
      <span class="status-chip">WEB2 BLOCKED</span>
    </div>
    <div class="top-right">
      <button type="button" class="top-btn mono" id="think-toggle" onclick="toggleThinking()">🧠 THINK: OFF</button>
      <a class="top-btn mono" href="bifrost://home/qr-sheet">🖨️ QR TAGS</a>
      <button type="button" class="top-btn mono" id="nodes-btn" onclick="openDrawer()">📡 NODES (0)</button>
    </div>
  </header>

  <nav class="quickbar mono" id="quickbar" aria-label="Direct Reticulum Links"></nav>

  <main class="stage" id="stage">
    <div class="hero-brand">
      <div class="brand-lockup">
        <img src="bifrost://home/icons/bifrost.png" alt="BIFROST" style="width:62px;height:62px;border-radius:16px;box-shadow:0 0 28px rgba(56,189,248,0.28);border:1px solid rgba(132,204,22,0.42);" />
        <div class="wordmark mono">BIFROST</div>
      </div>
      <div class="tagline mono">THE RETICULUM GHOST BROWSER · ZERO-IP SOVEREIGN CLIENT · 0 COOKIES · 0 DNS</div>
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
      <div id="peers-list" style="display:flex; flex-direction:column; gap:10px;"></div>
    </aside>
  </div>

<script>
const conversation = [];
let isBusy = false;
let enableThinking = false;
let llmHash = null;
let llmModel = 'qwen3-4b-npu';

function detectGhostUri(raw) {
  const v = (raw || '').trim();
  if (!v) return null;
  const hybrid = v.match(/#ip=([^\s]+)/i);
  if (hybrid && hybrid[1]) return 'ironpass://' + hybrid[1].replace(/^\/+/, '');
  if (/^(bifrost|citadel|ironpass):\/\//i.test(v)) return v;
  if (/^[0-9a-f]{32}(\/.*)?$/i.test(v)) return 'bifrost://' + (v.includes('/') ? v : v + '/');
  return null;
}

function updateOmniboxMode() {
  const input = document.getElementById('omni-input');
  const form = document.getElementById('omnibox-form');
  const icon = document.getElementById('omni-icon');
  const btn = document.getElementById('omni-submit');
  const isUri = Boolean(detectGhostUri(input.value));
  form.classList.toggle('mode-uri', isUri);
  icon.textContent = isUri ? '🔒' : '✦';
  if (!isBusy) btn.textContent = isUri ? 'OPEN URI →' : 'ASK AI ↵';
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
  if (uri) { window.location.href = uri; return; }

  input.value = '';
  appendMessage('user', raw);
  conversation.push({ role: 'user', content: raw });
  isBusy = true;
  document.getElementById('omni-submit').disabled = true;

  try {
    const r = await fetch('bifrost://home/api/ai-chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ model: llmModel, enable_thinking: enableThinking, messages: conversation.slice(-12) }),
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

async function refreshPeers() {
  try {
    const r = await fetch('bifrost://home/api/peers');
    const d = await r.json();
    llmHash = d.llm_hash || null;
    if (d.llm_model) llmModel = d.llm_model;
    const peers = d.peers || [];
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

    list.innerHTML = peers.map(p => `
      <div class="peer-card">
        <div style="display:flex; justify-content:space-between; align-items:center; gap:8px;">
          <h3 style="font-size:14px;">${escHtml(p.alias)}</h3>
          <span class="status-chip ${p.active ? 'online' : 'waiting'} mono" style="padding:3px 8px;font-size:10px;">${p.active ? 'X25519 LIVE' : 'LINKING'}</span>
        </div>
        <div class="hash-box mono">bifrost://${p.hash}/</div>
        <div class="stats-row mono">
          <span>Reqs: ${p.requests}</span>
          <span>Wire: ${(p.wire_bytes/1024).toFixed(1)} KB</span>
          <span>Chunks: ${p.chunks_rx}</span>
        </div>
        <div class="card-actions">
          <button class="btn-sm mono" onclick="window.location.href='bifrost://${p.hash}/'">OPEN GHOST SITE →</button>
          <button class="btn-sm amber mono" onclick="window.location.href='ironpass://${p.hash}/dpp/demo-01'">OPEN DEMO DPP + LEDGER</button>
        </div>
      </div>
    `).join('');
  } catch (err) {
    document.getElementById('peers-list').innerHTML = `<div class="peer-card"><div class="mono" style="font-size:12px;color:var(--danger);">Error: ${escHtml(err)}</div></div>`;
  }
}

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
