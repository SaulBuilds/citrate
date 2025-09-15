# Connecting GUI to Testnet - Quick Fix

## The Problem
The GUI DAG Explorer shows "Waiting for DAG data" and all values are 0.00 because:
1. The GUI has its own embedded node with its own storage
2. The DAG visualization reads from the embedded node's storage (not from external RPC)
3. The embedded node isn't syncing blocks from the testnet

## The Solution
The GUI's embedded node needs to sync blocks from the testnet node. Here's how:

### Method 1: In the GUI Console (Recommended)
Open the GUI's developer console (usually F12 or right-click → Inspect) and run:

```javascript
// Connect to testnet node as a peer
await window.__TAURI__.core.invoke('add_bootnode', { entry: '127.0.0.1:30303' });
await window.__TAURI__.core.invoke('connect_bootnodes');

// Start syncing
await window.__TAURI__.core.invoke('start_node');
```

### Method 2: Using the GUI Interface
1. Go to Settings/Network in the GUI
2. Add bootnode: `127.0.0.1:30303`
3. Enable "Network" toggle
4. Click "Connect to Peers"
5. The GUI should start syncing blocks

### Method 3: Force Sync Command
If the above doesn't work, run this in the GUI console:

```javascript
// Connect to external testnet RPC (different approach)
await window.__TAURI__.core.invoke('connect_to_external_testnet', { 
  rpc_url: 'http://localhost:8545' 
});
```

### Method 4: Join Testnet Command
The GUI has a built-in testnet join command:

```javascript
await window.__TAURI__.core.invoke('join_testnet', {
  network: 'testnet',
  bootnodes: ['127.0.0.1:30303'],
  clear_chain: false
});
```

## Verification
After connecting, you should see:
1. Peer count increase (should show 1 peer)
2. Block height start increasing from 0
3. DAG visualization populate with blocks
4. Statistics update with real values

## Why This Happens
The GUI was designed to run its own node, not just be a viewer. It needs to:
1. Connect to the testnet node as a peer (P2P port 30303)
2. Sync blocks to its local storage
3. Read from that storage to display the DAG

## Current State
- Testnet node: Running on port 8545 (RPC) and 30303 (P2P)
- GUI: Running but with empty storage
- Solution: Connect GUI node to testnet P2P network

## If Nothing Works
The GUI may be in "devnet" mode instead of "testnet" mode. Try:

```javascript
// Switch to testnet mode
await window.__TAURI__.core.invoke('switch_to_testnet');

// Then connect
await window.__TAURI__.core.invoke('join_testnet', {
  network: 'testnet',
  bootnodes: ['127.0.0.1:30303']
});
```

## Alternative: Direct RPC Mode
If syncing doesn't work, you can try using the GUI in RPC-only mode (no local node):

```javascript
// Stop embedded node and use external RPC
await window.__TAURI__.core.invoke('stop_node');
await window.__TAURI__.core.invoke('connect_to_external_testnet', { 
  rpc_url: 'http://localhost:8545' 
});
```

Note: This may limit some features since the DAG visualization expects local storage.