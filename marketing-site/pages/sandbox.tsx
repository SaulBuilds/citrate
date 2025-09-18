import { useState } from 'react';
import axios from 'axios';

type Call = { method: string; params: any[] };

export default function Sandbox() {
  const [endpoint, setEndpoint] = useState('http://localhost:8545');
  const [output, setOutput] = useState<string>('');
  const [contract, setContract] = useState('');
  const [method, setMethod] = useState('eth_blockNumber');
  const [params, setParams] = useState<string>('[]');
  const [safeMode, setSafeMode] = useState(true);

  const run = async () => {
    try {
      const parsed = JSON.parse(params || '[]');
      // Safe mode: allow only read-only methods
      const readOnly = [
        'eth_blockNumber','eth_getBlockByNumber','eth_getTransactionByHash','eth_getTransactionReceipt',
        'eth_call','eth_getBalance','eth_getCode','eth_chainId','lattice_getVerification','lattice_listVerifications'
      ];
      if (safeMode && !readOnly.includes(method)) {
        setOutput('Safe mode: only read-only methods are permitted.');
        return;
      }
      const { data } = await axios.post(endpoint, { jsonrpc:'2.0', id:1, method, params: parsed });
      setOutput(JSON.stringify(data, null, 2));
    } catch (e:any) {
      setOutput(`Error: ${e.message}`);
    }
  };

  return (
    <main style={{padding:'2rem',fontFamily:'Inter,ui-sans-serif'}}>
      <h1>SDK Sandbox</h1>
      <p>Safely test Lattice RPC and read-only contract calls. Safe mode blocks state-changing methods.</p>
      <div style={{display:'grid',gap:'.75rem', maxWidth:800}}>
        <label>Endpoint <input value={endpoint} onChange={e=>setEndpoint(e.target.value)} style={{width:'100%'}}/></label>
        <label>Method <input value={method} onChange={e=>setMethod(e.target.value)} style={{width:'100%'}}/></label>
        <label>Params (JSON) <textarea value={params} onChange={e=>setParams(e.target.value)} rows={3} style={{width:'100%'}}/></label>
        <label><input type="checkbox" checked={safeMode} onChange={e=>setSafeMode(e.target.checked)}/> Safe mode (read-only)</label>
        <button onClick={run} style={{padding:'.5rem 1rem'}}>Run</button>
        <pre style={{background:'#111', color:'#0f0', padding:'1rem', minHeight:'10rem'}}>{output}</pre>
      </div>
      <section style={{marginTop:'2rem'}}>
        <h2>Quick Helpers</h2>
        <button onClick={()=>{ setMethod('eth_blockNumber'); setParams('[]'); }} style={{marginRight:'.5rem'}}>eth_blockNumber</button>
        <button onClick={()=>{ setMethod('eth_call'); setParams(JSON.stringify([{ to: contract, data:'0x' }, 'latest'], null, 2)); }}>
          eth_call (set contract first)
        </button>
        <div style={{marginTop:'.5rem'}}>
          <label>Contract (0x...) <input value={contract} onChange={e=>setContract(e.target.value)} style={{width:'100%'}}/></label>
        </div>
      </section>
    </main>
  );
}

