# Citrate Network Modes: Devnet vs Testnet

This guide explains how to run from a clean genesis, switch between devnet and testnet, and verify everything is wired up.

## Modes

- Devnet (local):
  - `chain_id = 1337`
  - Fast blocks (2s), RPC `127.0.0.1:8545`
  - Data dirs: core `./.lattice-devnet`, GUI `./gui-data/devnet/chain`
  - Use for rapid local iteration.

- Testnet (public):
  - Choose a public `chain_id` (e.g., `42069`)
  - Block time ~5–10s
  - Bootstrap nodes configured and reachable
  - Data dirs: core `~/.lattice-testnet` (or `./.lattice-testnet`), GUI `./gui-data/testnet/chain`

## Reset from Genesis (0 → 1)

Use the reset script to wipe data, start nodes, and verify:

```bash
# Devnet reset
MODE=devnet ./scripts/reset.sh

# Testnet reset
MODE=testnet ./scripts/reset.sh
```

The script:
- Removes the correct data dirs for the selected mode
- Waits for RPC
- Prints block height and genesis hash and confirms height advances 0 → 1

## Core Node Configs

Two sample TOMLs are provided:

- `node/config/devnet.toml`
- `node/config/testnet.toml`

Run the node with a specific config:

```bash
# Devnet
lattice --config node/config/devnet.toml

# Testnet
lattice --config node/config/testnet.toml
```

## GUI Node Config

The GUI stores its own `NodeConfig` and maintains separate data dirs for each mode. To switch modes consistently:

1. Stop the GUI node (from the Dashboard).
2. Pick the desired mode in Settings (Devnet/Testnet) once implemented, or manually edit the GUI `NodeConfig` JSON to:
   - Set `network`, `consensus.chain_id`, and `mempool.chain_id`
   - Set `data_dir` (e.g., `./gui-data/devnet` or `./gui-data/testnet`)
   - Configure `bootnodes` for testnet
3. Start the GUI node.

## Acceptance Checks

- After reset, height starts at 0 and advances to 1.
- Sending a transaction shows up in the GUI Dashboard (mempool) and flips to confirmed with a receipt.
- The DAG explorer reflects the block containing the tx, and RPC `eth_getTransactionReceipt` returns the receipt.

If you see blocks producing but no receipts:
- Ensure the producer executes transactions via the executor and persists receipts.
- Ensure `require_valid_signature` is properly set for your environment.
- Double-check that `chain_id` and genesis are consistent across all peers.

