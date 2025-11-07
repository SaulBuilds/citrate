# Citrate Testnet Deployment Options

You have **3 options** for deploying your testnet. Choose based on what's easiest for you.

---

## ✅ Current Status

- **Testnet Node**: Running locally (PID 78774), fresh chain from genesis
- **DNS**: testnet-rpc.citrate.ai → 103.155.232.198 ✅
- **GUI App**: Built and ready at `target/release/bundle/macos/citrate-core.app`
- **GUI Zip**: `target/release/bundle/macos/citrate-core.app.zip` (9.7 MB)
- **Current GUI Config**: Connects to `http://localhost:8545`

---

## Option 1: Configure Your Frontier Router

**Difficulty**: Medium
**Cost**: Free
**Best for**: Running testnet from your Mac permanently

### Pros
- No monthly costs
- Full control
- Uses your domain (testnet-rpc.citrate.ai)

### Cons
- Requires router configuration
- Mac must stay running 24/7
- Dependent on home internet connection
- Power outage = testnet goes offline

### Steps

1. **Access Router**
   - Try: http://192.168.254.254 (most common for Frontier)
   - Or: http://192.168.4.1
   - Or: http://192.168.1.254

2. **Login**
   - Username: `admin`
   - Password: Check sticker on router bottom
   - Common defaults: `password`, `admin`, or blank

3. **Navigate to Port Forwarding**
   - Look for: "Firewall" → "Port Forwarding"
   - Or: "Advanced" → "Port Forwarding"
   - Or: "NAT/Gaming" → "Port Forwarding"

4. **Add 3 Rules**

**Rule 1 - RPC:**
```
Description: Citrate-RPC
External Port Start: 8545
External Port End: 8545
Internal IP: 192.168.4.223
Internal Port Start: 8545
Internal Port End: 8545
Protocol: TCP
Enable: ✓
```

**Rule 2 - WebSocket:**
```
Description: Citrate-WS
External Port Start: 8546
External Port End: 8546
Internal IP: 192.168.4.223
Internal Port Start: 8546
Internal Port End: 8546
Protocol: TCP
Enable: ✓
```

**Rule 3 - P2P:**
```
Description: Citrate-P2P
External Port Start: 30303
External Port End: 30303
Internal IP: 192.168.4.223
Internal Port Start: 30303
Internal Port End: 30303
Protocol: TCP
Enable: ✓
```

5. **Save and Test**
   ```bash
   # From another network (phone hotspot)
   curl -X POST http://testnet-rpc.citrate.ai:8545 \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
   ```

### Troubleshooting
- **Can't access router?** Contact Frontier support: 1-800-921-8101
- **Port forwarding not saving?** Try rebooting router
- **Still not working?** Check Mac firewall settings

---

## Option 2: Deploy to Cloud Server (RECOMMENDED)

**Difficulty**: Easy
**Cost**: $6/month
**Best for**: Professional, reliable deployment

### Pros
- ✅ **No router configuration needed**
- ✅ Always online (99.99% uptime)
- ✅ Independent of your Mac
- ✅ Professional setup
- ✅ Easy to scale
- ✅ I can help you set it up via Docker

### Cons
- Small monthly cost ($6-12/month)

### Quick Setup (5 minutes)

1. **Create DigitalOcean Account**
   - Go to: https://www.digitalocean.com
   - Sign up (get $200 credit for 60 days)

2. **Create Droplet**
   - Choose: Ubuntu 24.04 LTS
   - Plan: Basic $6/month (1GB RAM, 25GB SSD)
   - Datacenter: Choose closest to you
   - Add SSH key or use password

3. **Point DNS to Droplet**
   - Copy droplet IP (e.g., 1.2.3.4)
   - Update GoDaddy DNS:
     - testnet-rpc.citrate.ai → [droplet IP]

4. **Deploy Citrate**
   ```bash
   # SSH into droplet
   ssh root@[droplet-ip]

   # I'll provide you with Docker commands
   # OR we can deploy the binary directly
   ```

### I Can Help With
- Docker deployment
- Binary deployment
- Setting up systemd service
- SSL/HTTPS setup
- Monitoring

---

## Option 3: Local Development Setup (Current)

**Difficulty**: Easy
**Cost**: Free
**Best for**: Testing and development

### How It Works

Users run their own local testnet node alongside the GUI app:

**User Download Package:**
1. `citrate-node` binary
2. `citrate-core.app` (GUI)
3. Instructions:
   - Run: `./citrate-node --config testnet.toml`
   - Launch: `citrate-core.app`
   - GUI connects to localhost:8545 automatically

### Pros
- No server or router configuration
- Good for testing
- Everyone on same testnet (P2P connection)

### Cons
- Each user must run a node
- Not ideal for non-technical users
- More setup for users

### Distribution

```bash
# Package for users
tar -czf citrate-testnet-v0.1.0.tar.gz \
  target/release/citrate \
  target/release/bundle/macos/citrate-core.app \
  node/config/testnet.toml \
  README_TESTNET.md

# Users extract and run:
# 1. ./citrate --config testnet.toml
# 2. Open citrate-core.app
```

---

## My Recommendation

**For Initial Testing (Right Now):**
- Use Option 3 (Local) - test with a few users

**For Production Launch:**
- Use Option 2 (Cloud) - most reliable and professional
- Cost: $6/month
- Setup time: ~10 minutes
- I can help you deploy

**Alternative:**
- Use Option 1 (Router) if you want to avoid monthly costs
- But be aware: requires your Mac to run 24/7

---

## Current Files Ready for Distribution

### What You Have Now

**App Bundle:**
```
target/release/bundle/macos/citrate-core.app
```

**Zip for Download:**
```
target/release/bundle/macos/citrate-core.app.zip (9.7 MB)
```

**Node Binary:**
```
target/release/citrate (binary for running testnet)
```

### Test Locally Right Now

```bash
# Terminal 1: Start testnet
./target/release/citrate --config node/config/testnet.toml

# Terminal 2: Launch GUI
open target/release/bundle/macos/citrate-core.app
```

The GUI will connect to localhost:8545 automatically!

---

## What Do You Want to Do?

**Tell me which option you prefer:**

1. **"Option 1"** - I'll walk you through Frontier router configuration step-by-step
2. **"Option 2"** - I'll help you set up a DigitalOcean droplet (recommended)
3. **"Option 3"** - I'll help you package everything for local distribution

Or just say **"Let's test locally first"** and I'll help you verify everything works before deciding on deployment.
