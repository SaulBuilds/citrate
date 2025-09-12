## Lattice v3

### Network Modes & Reset

See `docs/NETWORK_MODES.md` for details on running devnet vs testnet, switching GUI modes, and resetting from genesis.

Quick reset examples:

```bash
# Devnet reset
MODE=devnet ./scripts/reset.sh

# Testnet reset
MODE=testnet ./scripts/reset.sh
```

Sample node configs are provided:

```
node/config/devnet.toml
node/config/testnet.toml
```

GUI mode templates:

```
gui/lattice-core/config/devnet.json
gui/lattice-core/config/testnet.json
```
