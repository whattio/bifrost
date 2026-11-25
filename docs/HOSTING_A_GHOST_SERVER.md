# Hosting a 0-Port Ghost Web & DPP Server on Linux

Turn any Debian or Ubuntu Linux server (from a compact micro-server to a rack AI node) into a **0-port Ghost Server** reachable exclusively inside **BIFROST**.

## 1. Bind Your Web / DPP App Strictly to Localhost
Never bind your application to `0.0.0.0`. Bind strictly to `127.0.0.1` (for example, our included demo server):
```bash
python3 examples/demo-vault/server.py
# Listens ONLY on http://127.0.0.1:8080
```

## 2. Lock Down All Inbound Ports with `nftables` (`[CITADEL-DROP]`)
Apply a default-drop inbound policy on your Linux server (keeping only loopback, established connections, and optional LAN SSH if needed):
```nft
table inet ghost_shield {
    chain input {
        type filter hook input priority 0; policy drop;
        iif "lo" accept
        ct state established,related accept
        # Optional: allow management SSH strictly from your admin subnet
        # ip saddr 192.168.1.0/24 tcp dport 22 accept
        log prefix "[CITADEL-DROP] " drop
    }
}
```
Verify from another machine with `nmap` — your server shows **0 open ports**.

## 3. Dial Outbound Over Encrypted Reticulum (`citadel-rns-rs`)
Install the clean-room MIT Rust daemon from [`whattio/citadel-rns-rs`](https://github.com/whattio/citadel-rns-rs):
```bash
cargo install --git https://github.com/whattio/citadel-rns-rs.git --bin citadel-vault-daemon

citadel-vault-daemon \
  --identity-name my-ghost-server \
  --upstream http://127.0.0.1:8080 \
  --rns-peer <OPERATOR_WORKSTATION_IP>:14290
```
The daemon prints its deterministic 128-bit destination hash (e.g. `a1b2c3...`).

## 4. Open Inside BIFROST
On your operator workstation:
```bash
bifrost-ghost-browser --rns-listen 0.0.0.0:14290 --allow-hash <YOUR_128_BIT_HASH>
```
Navigate to:
- `bifrost://<YOUR_128_BIT_HASH>/` — full zero-IP web application
- `ironpass://<YOUR_128_BIT_HASH>/dpp/demo-01` — local Rust Ed25519 verified DPP + hash-chained EPCIS 2.0 Ledger

## 5. Connecting a Local Sovereign AI Model (Optional)
Point a second `citadel-vault-daemon` at any local OpenAI-compatible inference server (Ollama on `:11434`, `llama-server` on `:8090`, or a Ventuno Q 40 TOPS NPU):
```bash
citadel-vault-daemon \
  --identity-name local-npu-llm \
  --upstream http://127.0.0.1:8090 \
  --rns-peer <OPERATOR_WORKSTATION_IP>:14290
```
Store any optional Bearer API key outside git (`chmod 600 ~/.config/bifrost/llm_api_key`) and launch BIFROST with `--llm-hash <LLM_HASH> --llm-model <MODEL_NAME>`.

## 6. Connecting Across the Internet (Self-Hosted Blind Reticulum Relay)
On a local LAN or over Sub-GHz LoRa radio, BIFROST and 0-port Ghost Servers connect directly peer-to-peer with zero relays. Across the public internet—where both sides sit behind home/corporate NAT routers or Starlink/5G CGNAT—both sides dial **outbound** to a stateless **Blind Reticulum Transport Relay** (`citadel-rns-relay` from [`whattio/citadel-rns-rs`](https://github.com/whattio/citadel-rns-rs)):

```text
[Remote BIFROST Browser] ──(outbound, 0 ports)──┐
                                                ▼
                                  [Blind citadel-rns-relay]
                                  (1 TCP port, 0 keys, 0 plaintext)
                                                ▲
[0-Port Ghost Linux Server] ──(outbound, 0 ports)┘
```

1. **Run `citadel-rns-relay` on ONE reachable Linux server** (a compact Debian/Ubuntu micro-server on a fiber connection with TCP `14290` forwarded, or a small VPS):
   ```bash
   cargo install --git https://github.com/whattio/citadel-rns-rs.git --bin citadel-rns-relay
   citadel-rns-relay --identity-name my-relay --listen 0.0.0.0:14290
   ```
   *(Multiple relays can peer with each other via repeatable `--peer <host>:14290` flags for multi-site resilience—only one relay needs an inbound router port.)*
2. **Point your 0-port Ghost Server outbound to the relay:**
   ```bash
   citadel-vault-daemon \
     --identity-name my-ghost-server \
     --upstream http://127.0.0.1:8080 \
     --rns-peer <RELAY_HOST>:14290
   ```
3. **Launch BIFROST anywhere in the world (0 open ports on the workstation too):**
   ```bash
   bifrost-ghost-browser --no-listen --rns-peer <RELAY_HOST>:14290
   ```

### Why a Blind Reticulum Relay Is Not a Cloudflare Tunnel
Unlike a corporate zero-trust tunnel (which terminates TLS at vendor servers, decrypts your traffic in vendor RAM, and logs DNS/HTTP metadata), `citadel-rns-relay` **holds zero vault keys, zero user identities, and zero plaintext**. Every `X25519 + Ed25519` Reticulum Link is encrypted end-to-end between your Ghost Server and BIFROST; you own the relay hardware and can swap relays anytime without changing a single `bifrost://<128-bit-hash>` address.
