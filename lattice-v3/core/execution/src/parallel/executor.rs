// lattice-v3/core/execution/src/parallel/executor.rs

use super::conflict::{ConflictScheduler, DefaultAccessSetExtractor};
use crate::executor::Executor;
use crate::types::TransactionReceipt;
use futures::future::join_all;
use lattice_consensus::types::{Block, Transaction};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::task;

/// Parallel executor for transaction batches
pub struct ParallelExecutor {
    conflict_scheduler: ConflictScheduler,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self {
            conflict_scheduler: ConflictScheduler::new(Box::new(DefaultAccessSetExtractor)),
        }
    }

    /// Execute a batch of transactions with parallel scheduling
    pub async fn execute_batch_with(
        &self,
        executor: Arc<Executor>,
        block: &Block,
        transactions: Vec<Transaction>,
    ) -> anyhow::Result<Vec<TransactionReceipt>> {
        // Schedule transactions into non-conflicting groups
        let groups = self.conflict_scheduler.schedule(transactions);

        // Record metrics
        #[cfg(feature = "metrics")]
        crate::api::metrics_server::record_parallel_groups(groups.len());

        // Execute groups in parallel
        let mut tasks = Vec::new();
        for group in groups {
            let executor = executor.clone();
            let block = block.clone();

            let task = task::spawn(async move {
                let mut receipts = Vec::new();
                for tx in group {
                    match executor.execute_transaction(&block, &tx).await {
                        Ok(receipt) => receipts.push(receipt),
                        Err(e) => {
                            // Log error but continue with other transactions
                            tracing::error!("Transaction execution failed: {:?}", e);
                        }
                    }
                }
                receipts
            });

            tasks.push(task);
        }

        // Collect all receipts
        let results = join_all(tasks).await;
        let mut all_receipts = Vec::new();

        for result in results {
            match result {
                Ok(receipts) => all_receipts.extend(receipts),
                Err(e) => {
                    tracing::error!("Task execution failed: {:?}", e);
                }
            }
        }

        Ok(all_receipts)
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Group transactions by sender for sequential execution
pub fn group_by_sender_sequential(transactions: &[Transaction]) -> Vec<Vec<Transaction>> {
    let mut sender_groups: BTreeMap<Vec<u8>, Vec<Transaction>> = BTreeMap::new();

    for tx in transactions {
        let sender_bytes = tx.from.0.to_vec();
        sender_groups
            .entry(sender_bytes)
            .or_default()
            .push(tx.clone());
    }

    // Sort each group by nonce
    for group in sender_groups.values_mut() {
        group.sort_by_key(|tx| tx.nonce);
    }

    sender_groups.into_values().collect()
}

/// Plan round-robin execution order across groups
pub fn plan_round_robin(groups: &[Vec<Transaction>]) -> Vec<Transaction> {
    let mut result = Vec::new();
    let mut indices = vec![0; groups.len()];

    loop {
        let mut added = false;
        for (i, group) in groups.iter().enumerate() {
            if indices[i] < group.len() {
                result.push(group[indices[i]].clone());
                indices[i] += 1;
                added = true;
            }
        }
        if !added {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_consensus::types::{Hash, PublicKey, Signature};

    fn make_test_tx(from: [u8; 32], nonce: u64) -> Transaction {
        Transaction {
            hash: Hash::default(),
            nonce,
            from: PublicKey::new(from),
            to: None,
            value: 100,
            gas_limit: 21000,
            gas_price: 1,
            data: vec![],
            signature: Signature::default(),
            tx_type: None,
        }
    }

    #[test]
    fn test_group_by_sender() {
        let txs = vec![
            make_test_tx([1; 32], 0),
            make_test_tx([2; 32], 0),
            make_test_tx([1; 32], 1),
            make_test_tx([2; 32], 1),
        ];

        let groups = group_by_sender_sequential(&txs);
        assert_eq!(groups.len(), 2);

        // Check nonce ordering
        for group in groups {
            for i in 1..group.len() {
                assert!(group[i].nonce >= group[i - 1].nonce);
            }
        }
    }

    #[test]
    fn test_round_robin() {
        let groups = vec![
            vec![make_test_tx([1; 32], 0), make_test_tx([1; 32], 1)],
            vec![make_test_tx([2; 32], 0)],
        ];

        let plan = plan_round_robin(&groups);
        assert_eq!(plan.len(), 3);

        // Should interleave: [1,0], [2,0], [1,1]
        assert_eq!(plan[0].from.0, [1; 32]);
        assert_eq!(plan[1].from.0, [2; 32]);
        assert_eq!(plan[2].from.0, [1; 32]);
    }
}
