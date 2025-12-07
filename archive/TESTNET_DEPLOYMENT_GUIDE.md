# Citrate Testnet Deployment Guide

## Overview
This guide explains how to deploy a public Citrate testnet node and distribute GUI clients that connect to it.

## Current Status ✅

### What's Working
- ✅ Testnet node runs on port 8545 without conflicts
- ✅ GUI app configured to connect to external RPC via `externalRpc` field
- ✅ No database lock errors when both run simultaneously
- ✅ DMG distributable created successfully

### Build Artifacts
- **GUI App Bundle**: `/Users/soleilklosowski/Downloads/citrate/citrate/target/release/bundle/macos/citrate-core.app`
- **DMG Installer**: `/Users/soleilklosowski/Downloads/citrate/citrate/target/release/bundle/dmg/citrate-core_0.1.0_aarch64.dmg`
- **Node Binary**: `/Users/soleilklosowski/Downloads/citrate/citrate/target/release/citrate`

## Architecture

```
┌─────────────────────────────────────────┐
│         Public Testnet Node             │
│  - Runs on your server                  │
│  - Listens on 0.0.0.0:8545 (RPC)       │
│  - Listens on 0.0.0.0:8546 (WebSocket) │
│  - Listens on 0.0.0.0:30303 (P2P)      │
└─────────────────┬───────────────────────┘
                  │
                  │ HTTP/WebSocket
                  │
        ┌─────────┴──────────┐
        │                    │
   ┌────▼─────┐       ┌──────▼────┐
   │ GUI App  │       │ CLI Wallet│
   │ (User 1) │       │  (User 2) │
   └──────────┘       └───────────┘
```

## Step 1: Deploy Public Testnet Node

### 1.1 Configure Testnet for Public Access

Edit `/Users/soleilklosowski/Downloads/citrate/citrate/node/config/testnet.toml`:

```toml
[network]
listen_addr = "0.0.0.0:30303"  # Already configured
bootstrap_nodes = []
max_peers = 50

[rpc]
enabled = true
listen_addr = "0.0.0.0:8545"   # Already configured - accessible from anywhere
ws_addr = "0.0.0.0:8546"       # Already configured
```

### 1.2 Start Testnet Node

```bash
cd /Users/soleilklosowski/Downloads/citrate/citrate
./target/release/citrate --config node/config/testnet.toml
```

Or run as a background service (recommended for production):

```bash
nohup ./target/release/citrate --config node/config/testnet.toml > testnet.log 2>&1 &
```

### 1.3 Setup Firewall Rules

Ensure these ports are open on your server:
- **8545 (RPC)**: For wallet/GUI connections
- **8546 (WebSocket)**: For real-time updates
- **30303 (P2P)**: For node-to-node communication

Example using `ufw` (Ubuntu):
```bash
sudo ufw allow 8545/tcp
sudo ufw allow 8546/tcp
sudo ufw allow 30303/tcp
```

### 1.4 Setup Domain (Optional but Recommended)

Instead of IP addresses, use a domain:
```
testnet-rpc.citrate.com -> your_server_ip:8545
testnet-ws.citrate.com  -> your_server_ip:8546
```

## Step 2: Configure GUI for Public Testnet

### 2.1 Update GUI Config for Production

Edit `gui/citrate-core/config/testnet.json`:

**For localhost testing** (current):
```json
{
  "externalRpc": "http://localhost:8545",
  "enableNetwork": false,
  "discovery": false
}
```

**For production deployment** (change to your public IP/domain):
```json
{
  "externalRpc": "http://YOUR_PUBLIC_IP:8545",
  "enableNetwork": false,
  "discovery": false
}
```

Or with domain:
```json
{
  "externalRpc": "https://testnet-rpc.citrate.com",
  "enableNetwork": false,
  "discovery": false
}
```

### 2.2 Rebuild GUI with Production Config

```bash
cd gui/citrate-core
npm run tauri build
```

The DMG will be created at:
```
target/release/bundle/dmg/citrate-core_0.1.0_aarch64.dmg
```

## Step 3: Distribute to Users

### 3.1 Distribution Options

**Option A: Direct Download**
- Host the DMG on your website
- Users download and install directly

**Option B: Release via GitHub**
- Create a GitHub Release
- Upload the DMG as a release asset
- Users download from GitHub Releases page

### 3.2 User Installation Steps

1. Download `citrate-core_0.1.0_aarch64.dmg`
2. Open the DMG file
3. Drag `citrate-core.app` to Applications folder
4. Launch the app - it will automatically connect to your testnet

## Step 4: CLI Wallet Configuration

Users can also connect via CLI wallet:

```bash
cargo run --bin citrate-wallet -- \\
  --rpc-url http://YOUR_PUBLIC_IP:8545 \\
  balance 0x...
```

## Monitoring & Maintenance

### Check Node Status
```bash
# Check if node is running
ps aux | grep citrate

# Check RPC is responding
curl -X POST http://localhost:8545 \\
  -H "Content-Type: application/json" \\
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# View logs
tail -f testnet.log
```

### Restart Node
```bash
# Find the process
ps aux | grep citrate

# Kill it
kill <PID>

# Restart
./target/release/citrate --config node/config/testnet.toml
```

## Configuration Summary

### Testnet Node (`node/config/testnet.toml`)
- **Chain ID**: 42069
- **Block Time**: 2 seconds
- **RPC Port**: 8545
- **WebSocket Port**: 8546
- **P2P Port**: 30303
- **Data Directory**: `.citrate-testnet`

### GUI App (`gui/citrate-core/config/testnet.json`)
- **External RPC**: `http://localhost:8545` (change for production)
- **Chain ID**: 42069 (must match node)
- **Network Enabled**: `false` (GUI doesn't run its own node)
- **Discovery**: `false` (GUI connects as client only)

## Troubleshooting

### Issue: GUI shows "Lock error"
**Solution**: GUI is trying to run embedded node. Ensure `externalRpc` is set in `testnet.json`

### Issue: GUI can't connect to testnet
**Checklist**:
1. ✅ Testnet node is running: `ps aux | grep citrate`
2. ✅ RPC port is accessible: `curl http://YOUR_IP:8545`
3. ✅ Firewall allows port 8545
4. ✅ `externalRpc` in `testnet.json` matches your node's URL

### Issue: Users can't sync
**Checklist**:
1. ✅ Node RPC is listening on `0.0.0.0:8545` (not `127.0.0.1`)
2. ✅ Firewall rules allow incoming connections
3. ✅ Users have correct RPC URL in their config

## Security Considerations

1. **HTTPS/TLS**: For production, use HTTPS with a reverse proxy (nginx/caddy)
2. **Rate Limiting**: Implement rate limiting to prevent abuse
3. **CORS**: Configure CORS if serving web clients
4. **Monitoring**: Set up monitoring and alerting (Prometheus/Grafana)

## Next Steps for Production

1. **Setup HTTPS**
   - Get SSL certificate (Let's Encrypt)
   - Configure reverse proxy (nginx/caddy)
   - Update GUI config to use `https://`

2. **Setup Monitoring**
   - Install Prometheus for metrics
   - Setup Grafana dashboards
   - Configure alerts

3. **Document for Users**
   - Write user guide for GUI installation
   - Create FAQ for common issues
   - Setup support channel (Discord/Telegram)

4. **Test Distribution**
   - Test DMG installation on clean macOS
   - Verify automatic connection to testnet
   - Test wallet creation and transactions

## Quick Start Commands

```bash
# Start testnet node
cd /Users/soleilklosowski/Downloads/citrate/citrate
./target/release/citrate --config node/config/testnet.toml

# In another terminal, verify it's running
curl -X POST http://localhost:8545 \\
  -H "Content-Type: application/json" \\
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Launch GUI (it will connect automatically)
open target/release/bundle/macos/citrate-core.app
```

## Files Modified in This Session

1. **gui/citrate-core/src-tauri/Cargo.toml** - Added `primitives` dependency
2. **gui/citrate-core/src-tauri/src/node/mod.rs:746** - Added `get_executor()` method
3. **gui/citrate-core/src-tauri/src/node/mod.rs:1787** - Changed default data_dir to `citrate-gui`
4. **gui/citrate-core/src-tauri/src/lib.rs:786-790** - Simplified eth_call implementation
5. **gui/citrate-core/config/testnet.json** - Added `externalRpc`, disabled embedded node

## Success Criteria ✅

- [x] GUI compiles without errors
- [x] GUI creates distributable DMG
- [x] Testnet node runs independently
- [x] GUI and testnet run simultaneously without conflicts
- [x] No database lock errors
- [ ] Change `externalRpc` to production URL
- [ ] Test with users on different machines
- [ ] Setup HTTPS for production
- [ ] Create user documentation
