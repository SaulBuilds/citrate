#!/usr/bin/env bash
# Dev reset script: stop local peers, clear wallets/keystore, remove chain data to force fresh genesis
# WARNING: Destructive. Dev/test use only.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OS_NAME=$(uname -s)

echo "[1/5] Stopping local scaffolded peers (if any)…"
if [[ -x "$ROOT_DIR/scripts/scaffold-local-peers.sh" ]]; then
  "$ROOT_DIR/scripts/scaffold-local-peers.sh" down || true
else
  echo "- scaffold-local-peers.sh not found or not executable; skipping"
fi

echo "[2/5] Clearing GUI wallet accounts.json and node config…"
rm_if_exists() { [[ -e "$1" ]] && { echo "- rm $1"; rm -rf "$1"; } || true; }

case "$OS_NAME" in
  Darwin)
    DATA_ROOT="$HOME/Library/Application Support"
    CONFIG_ROOT="$HOME/Library/Preferences"
    ;;
  Linux)
    DATA_ROOT="${XDG_DATA_HOME:-$HOME/.local/share}"
    CONFIG_ROOT="${XDG_CONFIG_HOME:-$HOME/.config}"
    ;;
  *)
    DATA_ROOT="${XDG_DATA_HOME:-$HOME/.local/share}"
    CONFIG_ROOT="${XDG_CONFIG_HOME:-$HOME/.config}"
    ;;
esac

# Wallet accounts file
rm_if_exists "$DATA_ROOT/citrate-core/accounts.json"

# GUI node config
rm_if_exists "$CONFIG_ROOT/citrate-core/config.json"

echo "[3/5] Removing GUI node data directories to force fresh genesis…"
# NodeManager default data_dir = data_dir()/"citrate" with subdir "chain"
rm_if_exists "$DATA_ROOT/lattice"

echo "[4/5] Removing standalone node data (if used)…"
rm_if_exists "$HOME/.lattice"

echo "[5/5] Clearing OS keychain entries for citrate-core wallets (best effort)…"
if command -v security >/dev/null 2>&1; then
  # macOS Keychain cleanup
  # Enumerate accounts for service "citrate-core" and delete wallet_* entries
  mapfile -t ACCOUNTS < <(security find-generic-password -s citrate-core 2>/dev/null | awk -F '="' '/acct/ {print $2}' | sed 's/"$//') || true
  if [[ ${#ACCOUNTS[@]} -gt 0 ]]; then
    for acct in "${ACCOUNTS[@]}"; do
      if [[ "$acct" == wallet_* ]]; then
        echo "- Deleting keychain entry: $acct"
        security delete-generic-password -s citrate-core -a "$acct" >/dev/null 2>&1 || true
      fi
    done
  else
    echo "- No keychain entries found for service citrate-core"
  fi
else
  echo "- 'security' CLI not found; if on Linux, clear secrets from your keyring manager (service=citrate-core, account=wallet_<address>)"
fi

echo "Reset complete. Next steps:"
echo "  1) Start 5 local peers (optional): $ROOT_DIR/scripts/scaffold-local-peers.sh up"
echo "  2) In the GUI, set Network=devnet, Enable Network, Save while node is stopped"
echo "  3) Create/import a new account (wallet entries will use the new keystore format)"
echo "  4) Start the node, Import local bootnodes (Settings), then Connect Bootnodes Now"
