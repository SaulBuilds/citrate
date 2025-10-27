"use client";
import React, { useEffect, useState } from 'react';
import axios from 'axios';

type Model = {
  id: string;
  owner: string;
  name: string;
  format?: string;
  timestamp?: string;
};

type ArtifactStatus = { provider: string; status: string }[];

export default function ModelsPage() {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState(true);
  const [selected, setSelected] = useState<string | null>(null);
  const [artifacts, setArtifacts] = useState<string[]>([]);
  const [statuses, setStatuses] = useState<Record<string, ArtifactStatus>>({});
  const [showModal, setShowModal] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const res = await axios.get('/api/models?limit=20');
        setModels(res.data.models || []);
      } catch (e) {
        console.error('Failed to load models', e);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const rpcCall = async (method: string, params: any) => {
    const res = await axios.post('/rpc', {
      jsonrpc: '2.0', method, params, id: Date.now()
    });
    if (res.data.error) throw new Error(res.data.error.message);
    return res.data.result;
  };

  const openArtifacts = async (modelIdHex: string) => {
    try {
      setSelected(modelIdHex);
      setShowModal(true);
      const cids: string[] = await rpcCall('citrate_listModelArtifacts', modelIdHex);
      setArtifacts(cids);
      // Fetch per-CID status
      const st: Record<string, ArtifactStatus> = {};
      for (const cid of cids) {
        try {
          const status = await rpcCall('citrate_getArtifactStatus', cid);
          // status can be array or object, normalize
          const arr = Array.isArray(status) ? status : (typeof status === 'string' ? JSON.parse(status) : []);
          st[cid] = arr;
        } catch (e) {
          st[cid] = [];
        }
      }
      setStatuses(st);
    } catch (e) {
      console.error('Failed to load artifacts', e);
    }
  };

  if (loading) return <div className="p-6">Loading models…</div>;

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">Models</h1>
      <table className="min-w-full text-sm">
        <thead>
          <tr className="text-left border-b">
            <th className="py-2">Name</th>
            <th className="py-2">Owner</th>
            <th className="py-2">Format</th>
            <th className="py-2">Artifacts</th>
          </tr>
        </thead>
        <tbody>
          {models.map((m: any, i) => (
            <tr key={i} className="border-b">
              <td className="py-2">{m.name || 'Model'}</td>
              <td className="py-2">{m.owner?.slice?.(0, 10)}…</td>
              <td className="py-2">{m.format || '-'}</td>
              <td className="py-2">
                {(() => {
                  const idHex: string | undefined = m.modelIdHex || m.modelId || m.id;
                  if (idHex && /^([0-9a-fA-F]{64})$/.test(idHex.replace(/^0x/, ''))) {
                    const hex = idHex.replace(/^0x/, '');
                    return (
                      <>
                        <button className="px-2 py-1 bg-blue-600 text-white rounded mr-2" onClick={() => openArtifacts(hex)}>View</button>
                        <a className="px-2 py-1 bg-gray-800 text-white rounded" href={`/models/${hex}`}>Details</a>
                      </>
                    );
                  }
                  return <span className="text-gray-400">N/A</span>;
                })()}
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {showModal && selected && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center">
          <div className="bg-white text-black rounded shadow-lg w-full max-w-2xl p-4">
            <div className="flex items-center justify-between mb-2">
              <h2 className="text-xl font-semibold">Artifacts for {selected.slice(0,8)}…</h2>
              <button className="px-2 py-1" onClick={() => setShowModal(false)}>Close</button>
            </div>
            {artifacts.length === 0 && <div className="text-gray-500">No artifacts found.</div>}
            {artifacts.map((cid) => (
              <div key={cid} className="border rounded p-2 mb-2">
                <div className="font-mono text-xs break-all">{cid}</div>
                <div className="mt-1">
                  {(statuses[cid] || []).length > 0 ? (
                    <ul className="list-disc ml-5 text-sm">
                      {(statuses[cid] || []).map((s, idx) => (
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
        </div>
      )}
    </div>
  );
}
