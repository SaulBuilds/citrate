"use client";
import React, { useEffect, useState } from 'react';
import axios from 'axios';

export default function ModelDetails({ params }: { params: { id: string } }) {
  const modelIdHex = params.id;
  const [model, setModel] = useState<any>(null);
  const [artifacts, setArtifacts] = useState<string[]>([]);
  const [statuses, setStatuses] = useState<Record<string, any[]>>({});
  const [loading, setLoading] = useState(true);

  const rpcCall = async (method: string, params: any) => {
    const res = await axios.post('/rpc', { jsonrpc: '2.0', method, params, id: Date.now() });
    if (res.data.error) throw new Error(res.data.error.message);
    return res.data.result;
  };

  useEffect(() => {
    (async () => {
      try {
        const m = await rpcCall('lattice_getModel', modelIdHex);
        setModel(m);
        const cids: string[] = await rpcCall('lattice_listModelArtifacts', modelIdHex);
        setArtifacts(cids);
        const st: Record<string, any[]> = {};
        for (const cid of cids) {
          try {
            const s = await rpcCall('lattice_getArtifactStatus', cid);
            const arr = Array.isArray(s) ? s : (typeof s === 'string' ? JSON.parse(s) : []);
            st[cid] = arr;
          } catch { st[cid] = []; }
        }
        setStatuses(st);
      } catch (e) {
        console.error('Failed to fetch model details', e);
      } finally {
        setLoading(false);
      }
    })();
  }, [modelIdHex]);

  if (loading) return <div className="p-6">Loading…</div>;

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">Model {modelIdHex.slice(0,8)}…</h1>
      {model ? (
        <div className="grid grid-cols-2 gap-4 mb-4">
          <div className="bg-gray-900 p-4 rounded">
            <div className="text-sm text-gray-400">Owner</div>
            <div className="font-mono text-xs break-all">{model.owner || 'unknown'}</div>
          </div>
          <div className="bg-gray-900 p-4 rounded">
            <div className="text-sm text-gray-400">Name</div>
            <div>{model.metadata?.name || model.name || 'Model'}</div>
          </div>
          <div className="bg-gray-900 p-4 rounded col-span-2">
            <div className="text-sm text-gray-400">Description</div>
            <div>{model.metadata?.description || '-'}</div>
          </div>
        </div>
      ) : (
        <div className="text-gray-500">Model not found.</div>
      )}

      <h2 className="text-xl font-semibold mt-6 mb-2">Artifacts</h2>
      {artifacts.length === 0 && <div className="text-gray-500">No artifacts.</div>}
      {artifacts.map((cid) => (
        <div key={cid} className="border rounded p-2 mb-2">
          <div className="font-mono text-xs break-all">{cid}</div>
          <div className="mt-1">
            {(statuses[cid] || []).length > 0 ? (
              <ul className="list-disc ml-5 text-sm">
                {(statuses[cid] || []).map((s: any, idx: number) => (
                  <li key={idx}><span className="font-medium">{s.provider}</span>: {s.status}</li>
                ))}
              </ul>
            ) : (
              <span className="text-gray-500">Status: unknown</span>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}

