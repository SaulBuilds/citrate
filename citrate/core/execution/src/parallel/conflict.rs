// citrate/core/execution/src/parallel/conflict.rs

// Conflict detection for parallel execution
use citrate_consensus::types::Transaction;
use std::collections::HashSet;

/// Represents read/write dependencies for a transaction
#[derive(Debug, Clone, Default)]
pub struct AccessSet {
    pub reads: HashSet<Vec<u8>>,
    pub writes: HashSet<Vec<u8>>,
}

impl AccessSet {
    /// Check if this access set conflicts with another
    pub fn conflicts_with(&self, other: &AccessSet) -> bool {
        // Write-Write conflict
        if !self.writes.is_disjoint(&other.writes) {
            return true;
        }
        // Read-Write conflict
        if !self.reads.is_disjoint(&other.writes) {
            return true;
        }
        // Write-Read conflict
        if !self.writes.is_disjoint(&other.reads) {
            return true;
        }
        false
    }

    /// Merge another access set into this one
    pub fn merge(&mut self, other: &AccessSet) {
        self.reads.extend(other.reads.iter().cloned());
        self.writes.extend(other.writes.iter().cloned());
    }
}

/// Extract access sets from transactions
pub trait AccessSetExtractor: Send + Sync {
    fn extract(&self, tx: &Transaction) -> AccessSet;
}

/// Default extractor based on transaction type
pub struct DefaultAccessSetExtractor;

impl AccessSetExtractor for DefaultAccessSetExtractor {
    fn extract(&self, tx: &Transaction) -> AccessSet {
        let mut access_set = AccessSet::default();

        // Sender always writes (nonce, balance)
        access_set
            .writes
            .insert(format!("account:{}", hex::encode(tx.from.0)).into_bytes());

        // Recipient reads/writes depend on transaction type
        if let Some(to) = &tx.to {
            let key = format!("account:{}", hex::encode(to.0)).into_bytes();
            access_set.reads.insert(key.clone());
            access_set.writes.insert(key);

            // If it's a contract call, add storage keys based on data
            if !tx.data.is_empty() {
                // Function selector is first 4 bytes
                if tx.data.len() >= 4 {
                    let selector = &tx.data[0..4];
                    let storage_key =
                        format!("storage:{}:{}", hex::encode(to.0), hex::encode(selector))
                            .into_bytes();
                    access_set.reads.insert(storage_key.clone());
                    access_set.writes.insert(storage_key);
                }
            }
        }

        // Model transactions access model registry
        if tx.data.len() > 32 && tx.data[0] == 0xA0 {
            access_set.reads.insert(b"registry:models".to_vec());
            access_set.writes.insert(b"registry:models".to_vec());
        }

        access_set
    }
}

/// Conflict-aware transaction scheduler
pub struct ConflictScheduler {
    extractor: Box<dyn AccessSetExtractor>,
}

impl ConflictScheduler {
    pub fn new(extractor: Box<dyn AccessSetExtractor>) -> Self {
        Self { extractor }
    }

    /// Schedule transactions into non-conflicting groups
    pub fn schedule(&self, transactions: Vec<Transaction>) -> Vec<Vec<Transaction>> {
        let mut groups: Vec<(Vec<Transaction>, AccessSet)> = Vec::new();

        for tx in transactions {
            let access_set = self.extractor.extract(&tx);
            let mut placed = false;

            // Try to place in an existing group
            for (group, group_access) in groups.iter_mut() {
                if !access_set.conflicts_with(group_access) {
                    group.push(tx.clone());
                    group_access.merge(&access_set);
                    placed = true;
                    break;
                }
            }

            // Create new group if needed
            if !placed {
                groups.push((vec![tx], access_set));
            }
        }

        groups.into_iter().map(|(txs, _)| txs).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use citrate_consensus::types::{Hash, PublicKey, Signature};

    fn make_test_tx(from: [u8; 32], to: Option<[u8; 32]>, nonce: u64) -> Transaction {
        Transaction {
            hash: Hash::new([1; 32]),
            nonce,
            from: PublicKey::new(from),
            to: to.map(PublicKey::new),
            value: 100,
            gas_limit: 21000,
            gas_price: 1,
            data: vec![],
            signature: Signature::new([0; 64]),
            tx_type: None,
        }
    }

    #[test]
    fn test_access_set_conflicts() {
        let mut set1 = AccessSet::default();
        set1.writes.insert(b"key1".to_vec());
        set1.reads.insert(b"key2".to_vec());

        let mut set2 = AccessSet::default();
        set2.writes.insert(b"key2".to_vec());

        assert!(set1.conflicts_with(&set2)); // Read-Write conflict
    }

    #[test]
    fn test_conflict_scheduler() {
        let scheduler = ConflictScheduler::new(Box::new(DefaultAccessSetExtractor));

        let tx1 = make_test_tx([1; 32], Some([2; 32]), 0);
        let tx2 = make_test_tx([3; 32], Some([4; 32]), 0);
        let tx3 = make_test_tx([1; 32], Some([5; 32]), 1); // Conflicts with tx1

        let groups = scheduler.schedule(vec![tx1, tx2, tx3]);

        assert_eq!(groups.len(), 2); // tx1 and tx3 conflict, so 2 groups
        assert_eq!(groups[0].len(), 2); // tx1 and tx2 can be parallel
        assert_eq!(groups[1].len(), 1); // tx3 must be separate
    }

    #[test]
    fn test_disjoint_senders_single_group() {
        let scheduler = ConflictScheduler::new(Box::new(DefaultAccessSetExtractor));

        // Three transactions with distinct sender/recipient pairs → no conflicts
        let tx1 = make_test_tx([1; 32], Some([2; 32]), 0);
        let tx2 = make_test_tx([3; 32], Some([4; 32]), 0);
        let tx3 = make_test_tx([5; 32], Some([6; 32]), 0);

        let groups = scheduler.schedule(vec![tx1, tx2, tx3]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }
}
