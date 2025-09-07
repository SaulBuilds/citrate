// Parallel execution scaffolding for Sprint 10

use lattice_consensus::types::{Transaction, PublicKey, Hash, Signature};
use crate::types::TransactionReceipt;
use crate::executor::Executor;
use tokio::task;
use futures::future::join_all;
use std::sync::Arc;
use lattice_consensus::types::Block;
use std::collections::BTreeMap;

/// Placeholder thread pool type alias (to be replaced by rayon/tokio tasks)
pub type ThreadPool = (); 

pub struct ConflictDetector;

impl ConflictDetector {
    pub fn new() -> Self { Self }
    /// Returns true if the two transactions have conflicting state access.
    pub fn conflicts(&self, _a: &Transaction, _b: &Transaction) -> bool { false }
}

pub struct ParallelExecutor {
    pub thread_pool: ThreadPool,
    pub conflict_detector: ConflictDetector,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self { thread_pool: (), conflict_detector: ConflictDetector::new() }
    }

    /// Execute a batch of transactions with conservative parallelism.
    /// NOTE: Sequential fallback until conflict detection and scheduling are implemented.
    pub async fn execute_batch(
        &self,
        transactions: Vec<Transaction>,
    ) -> anyhow::Result<Vec<TransactionReceipt>> {
        // TODO: Group non-conflicting txs; run in parallel tasks; maintain deterministic ordering per group.
        let _ = &self.thread_pool;
        let _ = &self.conflict_detector;
        let receipts: Vec<TransactionReceipt> = Vec::new();
        Ok(receipts)
    }

    /// Execute a batch using per-sender sequential groups scheduled in parallel.
    /// WARNING: Cross-sender conflicts are not detected yet; this is a first
    /// increment. Use in controlled environments until conflict detection is added.
    pub async fn execute_batch_with(
        &self,
        executor: Arc<Executor>,
        block: &Block,
        transactions: Vec<Transaction>,
    ) -> anyhow::Result<Vec<TransactionReceipt>> {
        // Group by sender with nonce ordering
        let groups = group_by_sender_sequential(&transactions);

        // Shared block for all tasks
        let block = Arc::new(block.clone());

        // Spawn one task per group; each task executes sequentially
        let mut handles = Vec::with_capacity(groups.len());
        for group in groups.iter() {
            let exec = executor.clone();
            let blk = block.clone();
            let g = group.clone();
            let handle = task::spawn(async move {
                let mut out: Vec<(Hash, TransactionReceipt)> = Vec::with_capacity(g.len());
                for tx in g.iter() {
                    match exec.execute_transaction(&blk, tx).await {
                        Ok(rcpt) => out.push((tx.hash, rcpt)),
                        Err(e) => {
                            // Failed receipts: push a failed receipt with minimal info
                            let failed = TransactionReceipt {
                                tx_hash: tx.hash,
                                block_hash: blk.hash(),
                                block_number: blk.header.height,
                                from: crate::types::Address::from_public_key(&tx.from),
                                to: tx.to.map(|pk| crate::types::Address::from_public_key(&pk)),
                                gas_used: 0,
                                status: false,
                                logs: vec![],
                                output: vec![],
                            };
                            out.push((tx.hash, failed));
                        }
                    }
                }
                out
            });
            handles.push(handle);
        }

        // Collect results
        let results = join_all(handles).await;
        let mut by_hash = std::collections::HashMap::new();
        for res in results.into_iter() {
            let pairs = res.map_err(|e| anyhow::anyhow!("task join error: {}", e))?;
            for (h, r) in pairs.into_iter() {
                by_hash.insert(h, r);
            }
        }

        // Flatten in deterministic round-robin plan order
        let plan = plan_round_robin(&groups);
        let mut receipts = Vec::with_capacity(plan.len());
        for tx in plan.into_iter() {
            if let Some(rcpt) = by_hash.remove(&tx.hash) {
                receipts.push(rcpt);
            }
        }

        Ok(receipts)
    }

    /// Execute a batch with naive cross-sender conflict avoidance.
    /// A key extractor maps a transaction to a set of conflict keys (bytes).
    /// Transactions that share any key will be placed in different stripes;
    /// each stripe runs as a sequential task in parallel with others.
    pub async fn execute_batch_with_conflicts<F>(
        &self,
        executor: Arc<Executor>,
        block: &Block,
        transactions: Vec<Transaction>,
        mut key_fn: F,
    ) -> anyhow::Result<Vec<TransactionReceipt>>
    where
        F: FnMut(&Transaction) -> Vec<Vec<u8>>,
    {
        // First, keep per-sender nonce ordering
        let groups = group_by_sender_sequential(&transactions);
        let plan = plan_round_robin(&groups);

        // Partition into stripes by conflict keys
        let mut stripes: Vec<(Vec<Transaction>, Vec<Vec<u8>>)> = Vec::new();
        'outer: for tx in plan.into_iter() {
            let keys = key_fn(&tx);
            // Try to place in an existing stripe with no key overlap
            for (stripe, stripe_keys) in stripes.iter_mut() {
                if !overlaps(&keys, stripe_keys) {
                    stripe.push(tx.clone());
                    stripe_keys.extend(keys);
                    continue 'outer;
                }
            }
            // Create a new stripe
            stripes.push((vec![tx], keys));
        }

        // Spawn a task per stripe
        let block = Arc::new(block.clone());
        let mut handles = Vec::with_capacity(stripes.len());
        for (stripe, _keys) in stripes.into_iter() {
            let exec = executor.clone();
            let blk = block.clone();
            let handle = task::spawn(async move {
                let mut out = Vec::with_capacity(stripe.len());
                for tx in stripe {
                    match exec.execute_transaction(&blk, &tx).await {
                        Ok(rcpt) => out.push(rcpt),
                        Err(_) => {
                            out.push(TransactionReceipt {
                                tx_hash: tx.hash,
                                block_hash: blk.hash(),
                                block_number: blk.header.height,
                                from: crate::types::Address::from_public_key(&tx.from),
                                to: tx.to.map(|pk| crate::types::Address::from_public_key(&pk)),
                                gas_used: 0,
                                status: false,
                                logs: vec![],
                                output: vec![],
                            });
                        }
                    }
                }
                out
            });
            handles.push(handle);
        }

        let results = join_all(handles).await;
        let mut receipts = Vec::new();
        for res in results.into_iter() {
            let mut part = res.map_err(|e| anyhow::anyhow!("task join error: {}", e))?;
            receipts.append(&mut part);
        }
        Ok(receipts)
    }
}

fn overlaps(keys: &Vec<Vec<u8>>, existing: &Vec<Vec<u8>>) -> bool {
    for k in keys {
        if existing.iter().any(|ek| ek == k) {
            return true;
        }
    }
    false
}

/// Default conflict key extractor: sender and optional recipient addresses.
pub fn default_conflict_keys(tx: &Transaction) -> Vec<Vec<u8>> {
    let mut v = Vec::with_capacity(2);
    v.push(tx.from.as_bytes().to_vec());
    if let Some(to) = &tx.to {
        v.push(to.as_bytes().to_vec());
    }
    v
}

/// Group transactions by sender and sort each group by nonce (ascending).
/// Deterministically orders groups by sender bytes for stable scheduling.
pub fn group_by_sender_sequential(txs: &[Transaction]) -> Vec<Vec<Transaction>> {
    // Use BTreeMap to have deterministic ordering by sender bytes
    let mut groups: BTreeMap<[u8; 32], Vec<&Transaction>> = BTreeMap::new();
    for tx in txs {
        groups
            .entry(tx.from.as_bytes().to_owned())
            .or_default()
            .push(tx);
    }

    let mut grouped: Vec<Vec<Transaction>> = Vec::with_capacity(groups.len());
    for (_sender, mut list) in groups {
        // Sort by nonce
        list.sort_by_key(|t| t.nonce);
        grouped.push(list.into_iter().cloned().collect());
    }
    grouped
}

/// Round-robin planner: interleave transactions across groups while preserving
/// per-group order. Returns a flattened, deterministic order.
pub fn plan_round_robin(groups: &[Vec<Transaction>]) -> Vec<Transaction> {
    if groups.is_empty() {
        return Vec::new();
    }
    let mut indices: Vec<usize> = vec![0; groups.len()];
    let mut remaining: usize = groups.iter().map(|g| g.len()).sum();
    let mut schedule: Vec<Transaction> = Vec::with_capacity(remaining);

    let mut gi = 0usize;
    while remaining > 0 {
        if groups[gi].len() > indices[gi] {
            schedule.push(groups[gi][indices[gi]].clone());
            indices[gi] += 1;
            remaining -= 1;
        }
        gi = (gi + 1) % groups.len();
    }
    schedule
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pk(byte: u8) -> PublicKey { PublicKey::new([byte; 32]) }
    fn sig() -> Signature { Signature::new([1; 64]) }

    fn mk_tx(sender: u8, nonce: u64, gas_price: u64) -> Transaction {
        // Create unique hash based on sender and nonce
        let mut hash_bytes = [0u8; 32];
        hash_bytes[0] = sender;
        hash_bytes[1..9].copy_from_slice(&nonce.to_le_bytes());
        hash_bytes[9..17].copy_from_slice(&gas_price.to_le_bytes());

        Transaction {
            hash: Hash::new(hash_bytes),
            nonce,
            from: pk(sender),
            to: None,
            value: 0,
            gas_limit: 21000,
            gas_price,
            data: vec![],
            signature: sig(),
        }
    }

    #[test]
    fn test_group_by_sender_orders_by_nonce() {
        let txs = vec![
            mk_tx(1, 2, 10),
            mk_tx(1, 0, 10),
            mk_tx(1, 1, 10),
        ];

        let groups = group_by_sender_sequential(&txs);
        assert_eq!(groups.len(), 1);
        let nonces: Vec<u64> = groups[0].iter().map(|t| t.nonce).collect();
        assert_eq!(nonces, vec![0, 1, 2]);
    }

    #[test]
    fn test_group_by_sender_multiple_senders() {
        let txs = vec![
            mk_tx(2, 0, 10),
            mk_tx(1, 0, 10),
            mk_tx(2, 1, 10),
            mk_tx(1, 1, 10),
        ];
        let groups = group_by_sender_sequential(&txs);
        // Deterministic sender order: lower sender byte first (1 then 2)
        assert_eq!(groups.len(), 2);
        assert!(groups[0].iter().all(|t| t.from == pk(1)));
        assert!(groups[1].iter().all(|t| t.from == pk(2)));
        let nonces0: Vec<u64> = groups[0].iter().map(|t| t.nonce).collect();
        let nonces1: Vec<u64> = groups[1].iter().map(|t| t.nonce).collect();
        assert_eq!(nonces0, vec![0, 1]);
        assert_eq!(nonces1, vec![0, 1]);
    }

    #[test]
    fn test_plan_round_robin_interleaves_groups() {
        let g1 = vec![mk_tx(1, 0, 10), mk_tx(1, 1, 10), mk_tx(1, 2, 10)];
        let g2 = vec![mk_tx(2, 0, 10), mk_tx(2, 1, 10)];
        let schedule = plan_round_robin(&[g1.clone(), g2.clone()]);
        // Expect order: 1(0),2(0),1(1),2(1),1(2)
        let expected = vec![g1[0].hash, g2[0].hash, g1[1].hash, g2[1].hash, g1[2].hash];
        let got: Vec<Hash> = schedule.into_iter().map(|t| t.hash).collect();
        assert_eq!(got, expected);
    }
}
