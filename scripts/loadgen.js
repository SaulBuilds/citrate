#!/usr/bin/env node
// Simple load generator for Lattice RPC
// Usage: NODE_RPC=http://127.0.0.1:8545 PRIVATE_KEY=0x... RATE=10 node scripts/loadgen.js

const RPC = process.env.NODE_RPC || 'http://127.0.0.1:8545';
const RATE = parseInt(process.env.RATE || '5', 10); // tx per second
const MODE = process.env.MODE || 'tx'; // 'tx' | 'call'
const TO = process.env.TO || '0x0000000000000000000000000000000000000000';

async function main() {
  let provider, wallet;
  if (MODE === 'tx') {
    const { Wallet, JsonRpcProvider } = require('ethers');
    provider = new JsonRpcProvider(RPC);
    const pk = process.env.PRIVATE_KEY;
    if (!pk) throw new Error('PRIVATE_KEY required for MODE=tx');
    wallet = new Wallet(pk, provider);
  }

  let sent = 0, failed = 0;
  console.log(`Starting loadgen MODE=${MODE} RATE=${RATE}/s RPC=${RPC}`);
  setInterval(async () => {
    try {
      if (MODE === 'tx') {
        const tx = await wallet.sendTransaction({ to: TO, value: 0n });
        await tx.wait();
      } else {
        // simple JSON-RPC call without dependencies
        await fetch(RPC, {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ jsonrpc: '2.0', id: Date.now(), method: 'eth_blockNumber', params: [] })
        }).then(r => r.json());
      }
      sent++;
    } catch (e) {
      failed++;
      if (failed % 10 === 0) console.error('errors:', failed, e.message);
    }
  }, 1000 / RATE);

  setInterval(async () => {
    let bn = 0;
    try {
      const resp = await fetch(RPC, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ jsonrpc: '2.0', id: 1, method: 'eth_blockNumber', params: [] })
      }).then(r => r.json());
      if (resp && resp.result) bn = parseInt(resp.result, 16) || 0;
    } catch {}
    console.log(`[stats] ok=${sent} err=${failed} latest=${bn}`);
    sent = 0; failed = 0;
  }, 5000);
}

main().catch((e) => { console.error(e); process.exit(1); });
