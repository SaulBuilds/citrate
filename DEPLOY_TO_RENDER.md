# Deploy Lattice V3 Testnet to Render.com

## Quick Deployment Steps

### 1. Log into Render
Go to https://render.com and sign in to your account.

### 2. Create New Blueprint
- Click **"New"** → **"Blueprint"**
- Connect to your GitHub account if not already connected
- Select repository: **`SaulBuilds/lattice-v3`**
- Select branch: **`ci/cd-updates`**
- Blueprint file: `lattice-v3/render.yaml`

### 3. Review Services
Render will show you the following services to be created:
- **lattice-node** (Web Service) - Main blockchain node - $7/month
- **lattice-ipfs** (Web Service) - IPFS storage - $7/month
- **lattice-explorer** (Static Site) - Block explorer - FREE
- **lattice-db** (PostgreSQL) - Database - $7/month
- **lattice-redis** (Redis) - Cache - FREE

**Total Cost**: ~$21/month (with free trial until tomorrow)

### 4. Apply Blueprint
- Click **"Apply"** to start deployment
- Render will automatically:
  - Build Docker images
  - Create databases
  - Set up networking
  - Configure environment variables

### 5. Monitor Deployment
- Go to your Render Dashboard
- Watch the build logs for each service
- Wait for all services to show **"Live"** status (takes ~5-10 minutes)

### 6. Access Your Testnet
Once deployed, you'll have URLs like:
- **Node RPC**: `https://lattice-node.onrender.com` (Port 8545)
- **IPFS Gateway**: `https://lattice-ipfs.onrender.com`
- **Block Explorer**: `https://lattice-explorer.onrender.com`

### 7. Verify Deployment
Test your RPC endpoint:
```bash
curl -X POST https://lattice-node.onrender.com \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

Expected response:
```json
{"jsonrpc":"2.0","id":1,"result":"0x1"}
```

## Configuration Details

### Testnet Settings
- **Network**: Testnet
- **Chain ID**: 31337
- **Consensus**: GhostDAG (k=18)
- **Block Time**: 1 second
- **Region**: Oregon (US West)

### Service Plans
All services are on **Starter tier** which is perfect for testnet:
- 0.5 CPU
- 512 MB RAM
- Auto-scaling disabled
- 50 GB disk for node data
- 100 GB disk for IPFS

### Environment Variables
Already configured in render.yaml:
- `NETWORK_MODE=testnet`
- `CHAIN_ID=31337`
- `RUST_LOG=info`
- `ENABLE_AI_INFERENCE=true`
- `ENABLE_IPFS_STORAGE=true`

## Troubleshooting

### Build Failures
If the lattice-node build fails:
1. Check build logs in Render dashboard
2. Verify the `ci/cd-updates` branch has the latest code
3. Ensure Dockerfile exists at `lattice-v3/Dockerfile`

### Services Won't Start
If services start but crash:
1. Check the service logs for errors
2. Verify environment variables are set correctly
3. Check that PostgreSQL and Redis are running first

### Can't Connect to RPC
If the RPC endpoint doesn't respond:
1. Wait for the service to show "Live" status
2. Check the health endpoint: `https://lattice-node.onrender.com/health`
3. Review the node logs for startup errors

## Next Steps After Deployment

1. **Upload AI Models to IPFS**
   ```bash
   # Use the IPFS API to upload models
   curl -F file=@model.safetensors https://lattice-ipfs.onrender.com/api/v0/add
   ```

2. **Fund Test Accounts**
   - Genesis accounts are pre-funded
   - Use faucet to fund additional accounts
   - Check balances via RPC

3. **Deploy Smart Contracts**
   ```bash
   forge script script/Deploy.s.sol \
     --rpc-url https://lattice-node.onrender.com \
     --broadcast
   ```

4. **Test Transactions**
   - Send test transactions via wallet
   - Verify in block explorer
   - Check transaction receipts

## Cost Optimization

### Free Tier Services
- Redis (free plan) - 25 MB cache
- Explorer (static site) - FREE
- **Total Savings**: ~$10/month

### Upgrade Path
When ready for production:
- **Standard tier**: $20/month - 1 CPU, 2 GB RAM
- **Pro tier**: $85/month - 2 CPU, 4 GB RAM
- Enable auto-scaling for high traffic

## Support

### Render Documentation
- https://render.com/docs/blueprint-spec
- https://render.com/docs/docker

### Lattice Documentation
- See `RENDER_DEPLOYMENT_GUIDE.md` for detailed guide
- Check `CLAUDE.md` for architecture details

### Common Issues
- **408 Timeout on push**: Large files in git history (already handled)
- **Build takes too long**: Normal for first build (~5-10 minutes)
- **Services won't connect**: Check they're in same region

## Manual Alternative (If Blueprint Fails)

If the blueprint deployment doesn't work, you can create services manually:

1. **Create PostgreSQL Database**
   - New → PostgreSQL
   - Name: `lattice-db`
   - Plan: Starter
   - Region: Oregon

2. **Create Redis**
   - New → Redis
   - Name: `lattice-redis`
   - Plan: Free
   - Region: Oregon

3. **Create Node Service**
   - New → Web Service
   - Connect to `SaulBuilds/lattice-v3`
   - Branch: `ci/cd-updates`
   - Runtime: Docker
   - Dockerfile path: `./lattice-v3/Dockerfile`
   - Add environment variables from render.yaml

4. **Create Explorer**
   - New → Static Site
   - Same repo and branch
   - Build command: `cd lattice-v3/explorer && npm install && npm run build`
   - Publish directory: `lattice-v3/explorer/out`

---

## Ready to Deploy! 🚀

You're all set! Just go to https://render.com and follow the steps above.

The testnet will be live in ~10 minutes after clicking "Apply".
