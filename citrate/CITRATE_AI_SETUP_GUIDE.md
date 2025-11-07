# Citrate.ai Testnet Public Deployment Setup

## Current Status
- Fresh testnet node running (PID 78774)
- Chain restarted from genesis (block 0)
- Node listening on 0.0.0.0:8545 (RPC), 0.0.0.0:8546 (WebSocket), 0.0.0.0:30303 (P2P)
- GUI configured to connect to testnet-rpc.citrate.ai:8545

## Your Network Details
- **Public IP**: 103.155.232.198
- **Domain**: citrate.ai
- **Testnet RPC Subdomain**: testnet-rpc.citrate.ai

---

## Step 1: Configure GoDaddy DNS

### Login to GoDaddy
1. Go to https://dcc.godaddy.com/manage/citrate.ai/dns
2. Login to your GoDaddy account

### Add DNS A Record
1. Click "Add" button
2. Create the following A record:

```
Type: A
Name: testnet-rpc
Data: 103.155.232.198
TTL: 600 seconds (10 minutes)
```

3. Click "Save"

### DNS Propagation
- DNS changes typically propagate within 10-30 minutes
- You can check propagation status at: https://dnschecker.org/#A/testnet-rpc.citrate.ai

---

## Step 2: Configure Router Port Forwarding

You need to forward these ports from your router to your Mac's local IP address:

### Find Your Mac's Local IP
```bash
# Run this on your Mac to find local IP
ipconfig getifaddr en0
# or if using WiFi:
ipconfig getifaddr en1
```

### Router Configuration
Login to your router admin panel (usually http://192.168.1.1 or http://192.168.0.1)

**Create these port forwarding rules:**

| Service Name | External Port | Internal Port | Internal IP | Protocol |
|--------------|---------------|---------------|-------------|----------|
| Citrate RPC  | 8545          | 8545          | [Your Mac Local IP] | TCP |
| Citrate WS   | 8546          | 8546          | [Your Mac Local IP] | TCP |
| Citrate P2P  | 30303         | 30303         | [Your Mac Local IP] | TCP |

**Example Configuration:**
```
External Port: 8545
Internal IP: 192.168.1.100 (your Mac's local IP)
Internal Port: 8545
Protocol: TCP
```

### Common Router Admin URLs:
- Netgear: http://192.168.1.1
- TP-Link: http://192.168.0.1
- Linksys: http://192.168.1.1
- ASUS: http://192.168.1.1

---

## Step 3: Test External Access

### Wait for DNS Propagation
After adding the DNS record, wait 10-30 minutes and verify:

```bash
# Check DNS resolution
nslookup testnet-rpc.citrate.ai

# Should return:
# Name: testnet-rpc.citrate.ai
# Address: 103.155.232.198
```

### Test RPC Access
Once DNS is propagated and port forwarding is active:

```bash
# Test from external network (use phone hotspot or ask a friend)
curl -X POST http://testnet-rpc.citrate.ai:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Should return current block number:
# {"jsonrpc":"2.0","result":"0xXX","id":1}
```

### Local Testing (Before DNS/Port Forwarding)
You can test locally first:

```bash
# From your Mac
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

---

## Step 4: Rebuild GUI with Production Config

### Navigate to GUI Directory
```bash
cd /Users/soleilklosowski/Downloads/citrate/citrate/gui/citrate-core
```

### Rebuild Tauri App
```bash
npm run tauri build
```

This will create:
- **App Bundle**: `../../target/release/bundle/macos/citrate-core.app`
- **DMG Installer**: `../../target/release/bundle/dmg/citrate-core_0.1.0_aarch64.dmg`

---

## Step 5: Test GUI Connection

### Test Locally First
```bash
# Open the app
open ../../target/release/bundle/macos/citrate-core.app
```

The GUI should now attempt to connect to testnet-rpc.citrate.ai:8545

**Note**: The GUI will fail to connect until DNS and port forwarding are configured!

---

## Step 6: Distribute to Users

Once everything is working:

### Option A: Direct Download
1. Host the DMG on your website
2. Share download link: `https://citrate.ai/downloads/citrate-core_0.1.0_aarch64.dmg`

### Option B: GitHub Release
1. Create GitHub release: `git tag v0.1.0 && git push origin v0.1.0`
2. Upload DMG to release assets
3. Share GitHub release link

### User Instructions
Send users this:

```
1. Download citrate-core_0.1.0_aarch64.dmg
2. Open the DMG file
3. Drag citrate-core.app to Applications folder
4. Launch Citrate Core
5. The app will automatically connect to testnet-rpc.citrate.ai
6. Create or import your wallet
7. Start using the Citrate testnet!
```

---

## Monitoring & Maintenance

### Check Node Status
```bash
# Check if node is running
ps aux | grep citrate | grep testnet

# View logs
tail -f /Users/soleilklosowski/Downloads/citrate/citrate/testnet.log

# Check current block
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

### Restart Node
```bash
# Find PID
ps aux | grep citrate | grep testnet

# Kill it
kill [PID]

# Start fresh
cd /Users/soleilklosowski/Downloads/citrate/citrate
./target/release/citrate --config node/config/testnet.toml > testnet.log 2>&1 &
```

### Keep Mac Awake
Your Mac must stay running for the testnet to remain accessible:

```bash
# Prevent sleep while testnet runs
caffeinate -i &

# Or use macOS System Preferences:
# System Preferences > Energy Saver > Prevent computer from sleeping automatically
```

---

## Troubleshooting

### DNS Not Resolving
```bash
# Check if DNS is propagated
dig testnet-rpc.citrate.ai

# If not resolved after 1 hour, check GoDaddy DNS settings
```

### Connection Timeout from Outside
**Checklist:**
1. ✅ Node is running: `ps aux | grep citrate`
2. ✅ Port forwarding configured in router
3. ✅ Mac firewall allows incoming connections on ports 8545, 8546, 30303
4. ✅ DNS resolves to correct IP: `nslookup testnet-rpc.citrate.ai`

### Mac Firewall Configuration
```bash
# Check firewall status
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# Allow incoming connections for citrate
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /Users/soleilklosowski/Downloads/citrate/citrate/target/release/citrate
```

### GUI Can't Connect
1. Check DNS: `nslookup testnet-rpc.citrate.ai`
2. Check RPC from outside: Ask friend to test `curl http://testnet-rpc.citrate.ai:8545`
3. Check GUI config: `cat gui/citrate-core/config/testnet.json | grep externalRpc`
4. Rebuild GUI with correct config

---

## Security Considerations

### For Production Use (Future)
1. **Enable HTTPS**: Use nginx/caddy reverse proxy with Let's Encrypt SSL
2. **Rate Limiting**: Prevent abuse with request rate limits
3. **Firewall**: Only expose necessary ports
4. **Monitoring**: Set up Prometheus/Grafana for alerting
5. **Backups**: Regular backups of `.citrate-testnet` directory

### Current Setup (Testing)
- HTTP only (no encryption)
- No rate limiting
- Publicly accessible RPC
- **This is fine for testnet, NOT for mainnet**

---

## Quick Reference

### Node Commands
```bash
# Start testnet node
cd /Users/soleilklosowski/Downloads/citrate/citrate
./target/release/citrate --config node/config/testnet.toml > testnet.log 2>&1 &

# Stop testnet node
pkill -f "citrate.*testnet"

# View logs
tail -f testnet.log

# Check block number
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

### GUI Commands
```bash
# Rebuild GUI
cd gui/citrate-core
npm run tauri build

# Open GUI (for testing)
open ../../target/release/bundle/macos/citrate-core.app
```

### Files & Directories
- **Node Binary**: `target/release/citrate`
- **Testnet Config**: `node/config/testnet.toml`
- **Chain Data**: `.citrate-testnet/`
- **Node Logs**: `testnet.log`
- **GUI Config**: `gui/citrate-core/config/testnet.json`
- **GUI App**: `target/release/bundle/macos/citrate-core.app`
- **DMG Installer**: `target/release/bundle/dmg/citrate-core_0.1.0_aarch64.dmg`

---

## Next Steps

1. **Configure GoDaddy DNS** (10 minutes)
2. **Configure Router Port Forwarding** (10 minutes)
3. **Wait for DNS Propagation** (10-30 minutes)
4. **Test External Access** (5 minutes)
5. **Rebuild GUI** (3-5 minutes)
6. **Test GUI Connection** (5 minutes)
7. **Distribute DMG to Users** (ongoing)

**Total Setup Time**: ~1 hour (mostly waiting for DNS)

---

## Support

If you encounter issues:
1. Check this guide's Troubleshooting section
2. Review testnet.log for errors
3. Test RPC locally first, then externally
4. Verify DNS and port forwarding configuration
