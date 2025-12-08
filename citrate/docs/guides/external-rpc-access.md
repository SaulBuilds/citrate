# External RPC Access Guide

This guide covers how to expose your Citrate node's RPC endpoint for external access using ngrok or production reverse proxies.

## Quick Start with ngrok

### Prerequisites

1. Install ngrok:
   ```bash
   # macOS
   brew install ngrok

   # Linux
   curl -s https://ngrok-agent.s3.amazonaws.com/ngrok.asc | sudo tee /etc/apt/trusted.gpg.d/ngrok.asc >/dev/null
   echo "deb https://ngrok-agent.s3.amazonaws.com buster main" | sudo tee /etc/apt/sources.list.d/ngrok.list
   sudo apt update && sudo apt install ngrok

   # Windows (via chocolatey)
   choco install ngrok
   ```

2. Sign up at [ngrok.com](https://ngrok.com) and get your authtoken

3. Configure ngrok:
   ```bash
   ngrok config add-authtoken YOUR_AUTH_TOKEN
   ```

### Exposing the RPC Endpoint

1. Start your Citrate node:
   ```bash
   cargo run --release --bin citrate-node -- devnet
   # or
   ./citrate-node --config node/config/testnet.toml
   ```

2. In another terminal, start ngrok:
   ```bash
   # Expose JSON-RPC (HTTP)
   ngrok http 8545

   # Or expose with a static domain (paid plan)
   ngrok http --domain=your-subdomain.ngrok-free.app 8545
   ```

3. ngrok will display your public URL:
   ```
   Forwarding  https://abc123.ngrok-free.app -> http://localhost:8545
   ```

4. Test the connection:
   ```bash
   curl -X POST https://abc123.ngrok-free.app \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
   ```

### Exposing WebSocket Endpoint

For real-time subscriptions, expose the WebSocket port:

```bash
# In a separate terminal
ngrok http 8546
```

## RPC Endpoint Matrix

| Environment | HTTP URL | WebSocket URL | Notes |
|-------------|----------|---------------|-------|
| Local Dev | `http://localhost:8545` | `ws://localhost:8546` | Default |
| ngrok (temp) | `https://xxx.ngrok-free.app` | `wss://xxx.ngrok-free.app` | External testing |
| Production | `https://api.citrate.ai` | `wss://api.citrate.ai` | When deployed |

## Production Setup

For production deployments, use a proper reverse proxy instead of ngrok.

### Nginx Configuration

```nginx
upstream citrate_rpc {
    server 127.0.0.1:8545;
}

upstream citrate_ws {
    server 127.0.0.1:8546;
}

server {
    listen 443 ssl http2;
    server_name api.citrate.ai;

    ssl_certificate /etc/letsencrypt/live/api.citrate.ai/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.citrate.ai/privkey.pem;

    # JSON-RPC endpoint
    location / {
        proxy_pass http://citrate_rpc;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # CORS headers
        add_header Access-Control-Allow-Origin *;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
        add_header Access-Control-Allow-Headers "Content-Type";
    }

    # WebSocket endpoint
    location /ws {
        proxy_pass http://citrate_ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }
}
```

### Caddy Configuration

```caddyfile
api.citrate.ai {
    # JSON-RPC
    reverse_proxy localhost:8545

    # WebSocket (on /ws path)
    handle /ws {
        reverse_proxy localhost:8546
    }

    # CORS
    header Access-Control-Allow-Origin *
    header Access-Control-Allow-Methods "GET, POST, OPTIONS"
}
```

## Security Considerations

### Rate Limiting

For production, implement rate limiting:

```nginx
# In nginx http block
limit_req_zone $binary_remote_addr zone=rpc_limit:10m rate=100r/s;

# In server block
location / {
    limit_req zone=rpc_limit burst=200 nodelay;
    proxy_pass http://citrate_rpc;
}
```

### Method Filtering

Consider filtering dangerous RPC methods in production:

```nginx
location / {
    # Block admin methods
    if ($request_body ~* "admin_") {
        return 403;
    }
    if ($request_body ~* "debug_") {
        return 403;
    }

    proxy_pass http://citrate_rpc;
}
```

### Firewall Rules

```bash
# Allow only localhost access to RPC ports
sudo ufw allow from 127.0.0.1 to any port 8545
sudo ufw allow from 127.0.0.1 to any port 8546

# Allow nginx/caddy to access
sudo ufw allow 443/tcp
```

## GUI Configuration

To connect the Citrate GUI to a remote node:

1. Open Settings in the GUI
2. Under "RPC Endpoint", enter your ngrok or production URL
3. Click "Test Connection" to verify

Or modify `gui/citrate-core/config/testnet.json`:

```json
{
  "externalRpc": "https://your-ngrok-url.ngrok-free.app",
  "mempool": {
    "chainId": 7331
  }
}
```

**Current Testnet ChainId**: 7331

## Troubleshooting

### ngrok Connection Issues

1. **"Tunnel not found"**: Your ngrok session expired. Restart ngrok.
2. **"Too many connections"**: Free tier limit. Upgrade or wait.
3. **CORS errors**: Add CORS headers to your responses or use ngrok's `--host-header` flag.

### RPC Not Responding

1. Verify node is running: `curl http://localhost:8545`
2. Check node logs for errors
3. Ensure firewall isn't blocking localhost

### WebSocket Disconnections

1. Check proxy timeout settings (should be high for WS)
2. Verify `Connection: upgrade` header is preserved
3. Test with: `wscat -c wss://your-url.ngrok-free.app`

## Monitoring

### Check ngrok Status

```bash
# View active tunnels
curl http://localhost:4040/api/tunnels

# View request inspector
open http://localhost:4040
```

### Node Health Check

```bash
# Check sync status
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_syncing","params":[],"id":1}'

# Check peer count
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}'
```
