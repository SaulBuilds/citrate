---
title: JavaScript SDK
---

The JS SDK lives under `citrate/sdk/javascript`. Example (ContractManager):

```ts
import axios from 'axios';
import { ContractManager } from './contract';

const rpc = axios.create({ baseURL: 'http://localhost:8545' });
const cm = new ContractManager(rpc, { defaultAccount: '0x...', gasLimit: 3_000_000, gasPrice: '0x3b9aca00' });

// Read contract state
const value = await cm.read('0xContract', ABI, 'get', []);
```

