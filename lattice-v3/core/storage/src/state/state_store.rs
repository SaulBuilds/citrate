// lattice-v3/core/storage/src/state/state_store.rs

use crate::db::{column_families::*, RocksDB};
use anyhow::Result;
use lattice_consensus::types::Hash;
use lattice_execution::executor::StateStoreTrait;
use lattice_execution::types::{AccountState, Address, JobId, ModelId, ModelState, TrainingJob};
use std::sync::Arc;
use tracing::{debug, info};

/// State storage manager
pub struct StateStore {
    db: Arc<RocksDB>,
}

impl StateStoreTrait for StateStore {
    fn put_account(&self, address: &Address, account: &AccountState) -> Result<()> {
        let account_bytes = bincode::serialize(account)?;
        self.db.put_cf(CF_ACCOUNTS, &address.0, &account_bytes)?;
        debug!("Stored account state for {}", address);
        Ok(())
    }

    fn get_account(&self, address: &Address) -> Result<Option<AccountState>> {
        match self.db.get_cf(CF_ACCOUNTS, &address.0)? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    fn put_code(&self, code_hash: &Hash, code: &[u8]) -> Result<()> {
        self.db.put_cf(CF_CODE, code_hash.as_bytes(), code)?;
        debug!("Stored contract code with hash {}", code_hash);
        Ok(())
    }
}

impl StateStore {
    pub fn new(db: Arc<RocksDB>) -> Self {
        Self { db }
    }

    /// Store account state
    pub fn put_account(&self, address: &Address, account: &AccountState) -> Result<()> {
        let account_bytes = bincode::serialize(account)?;
        self.db.put_cf(CF_ACCOUNTS, &address.0, &account_bytes)?;
        debug!("Stored account state for {}", address);
        Ok(())
    }

    /// Get account state
    pub fn get_account(&self, address: &Address) -> Result<Option<AccountState>> {
        match self.db.get_cf(CF_ACCOUNTS, &address.0)? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Get all accounts (for state root calculation)
    pub fn get_all_accounts(&self) -> Result<Vec<(Address, AccountState)>> {
        let mut accounts = Vec::new();
        let iter = self.db.iter_cf(CF_ACCOUNTS)?;

        for (key, value) in iter {
            if key.len() == 20 {
                let mut addr_bytes = [0u8; 20];
                addr_bytes.copy_from_slice(&key);
                let address = Address(addr_bytes);
                let account: AccountState = bincode::deserialize(&value)?;
                accounts.push((address, account));
            }
        }

        Ok(accounts)
    }

    /// Get all storage (for state root calculation)
    pub fn get_all_storage(&self) -> Result<Vec<((Address, Hash), Hash)>> {
        let mut storage = Vec::new();
        let iter = self.db.iter_cf(CF_STORAGE)?;

        for (key, value) in iter {
            // Key format: address(20) + storage_key(32) = 52 bytes
            if key.len() == 52 {
                let mut addr_bytes = [0u8; 20];
                addr_bytes.copy_from_slice(&key[..20]);
                let address = Address(addr_bytes);

                let mut storage_key = [0u8; 32];
                storage_key.copy_from_slice(&key[20..52]);

                let mut storage_value = [0u8; 32];
                if value.len() >= 32 {
                    storage_value.copy_from_slice(&value[..32]);
                }

                storage.push(((address, Hash::new(storage_key)), Hash::new(storage_value)));
            }
        }

        Ok(storage)
    }

    /// Delete account state
    pub fn delete_account(&self, address: &Address) -> Result<()> {
        self.db.delete_cf(CF_ACCOUNTS, &address.0)?;
        Ok(())
    }

    /// Store contract storage value
    pub fn put_storage(&self, address: &Address, key: &[u8], value: &[u8]) -> Result<()> {
        let storage_key = storage_key(address, key);
        self.db.put_cf(CF_STORAGE, &storage_key, value)?;
        Ok(())
    }

    /// Get contract storage value
    pub fn get_storage(&self, address: &Address, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let storage_key = storage_key(address, key);
        self.db.get_cf(CF_STORAGE, &storage_key)
    }

    /// Delete contract storage value
    pub fn delete_storage(&self, address: &Address, key: &[u8]) -> Result<()> {
        let storage_key = storage_key(address, key);
        self.db.delete_cf(CF_STORAGE, &storage_key)?;
        Ok(())
    }

    /// Store contract code
    pub fn put_code(&self, code_hash: &Hash, code: &[u8]) -> Result<()> {
        self.db.put_cf(CF_CODE, code_hash.as_bytes(), code)?;
        debug!("Stored contract code with hash {}", code_hash);
        Ok(())
    }

    /// Get contract code
    pub fn get_code(&self, code_hash: &Hash) -> Result<Option<Vec<u8>>> {
        self.db.get_cf(CF_CODE, code_hash.as_bytes())
    }

    /// Store model state
    pub fn put_model(&self, model_id: &ModelId, model: &ModelState) -> Result<()> {
        let model_bytes = bincode::serialize(model)?;
        self.db
            .put_cf(CF_MODELS, model_id.0.as_bytes(), &model_bytes)?;

        // Index by owner
        let owner_key = owner_model_key(&model.owner, model_id);
        self.db.put_cf(CF_METADATA, &owner_key, &[])?;

        debug!("Stored model {:?}", model_id);
        Ok(())
    }

    /// Get model state
    pub fn get_model(&self, model_id: &ModelId) -> Result<Option<ModelState>> {
        match self.db.get_cf(CF_MODELS, model_id.0.as_bytes())? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Get models by owner
    pub fn get_models_by_owner(&self, owner: &Address) -> Result<Vec<ModelId>> {
        let prefix = owner_model_prefix(owner);
        let mut model_ids = Vec::new();

        for (key, _) in self.db.prefix_iter_cf(CF_METADATA, &prefix)? {
            if key.len() > prefix.len() {
                let model_hash_bytes = &key[prefix.len()..];
                if model_hash_bytes.len() == 32 {
                    let mut hash_array = [0u8; 32];
                    hash_array.copy_from_slice(model_hash_bytes);
                    model_ids.push(ModelId(Hash::new(hash_array)));
                }
            }
        }

        Ok(model_ids)
    }

    /// Store training job
    pub fn put_training_job(&self, job: &TrainingJob) -> Result<()> {
        let job_bytes = bincode::serialize(job)?;
        self.db
            .put_cf(CF_TRAINING, job.id.0.as_bytes(), &job_bytes)?;

        // Index by owner
        let owner_key = owner_job_key(&job.owner, &job.id);
        self.db.put_cf(CF_METADATA, &owner_key, &[])?;

        debug!("Stored training job {:?}", job.id);
        Ok(())
    }

    /// Get training job
    pub fn get_training_job(&self, job_id: &JobId) -> Result<Option<TrainingJob>> {
        match self.db.get_cf(CF_TRAINING, job_id.0.as_bytes())? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Store state root for a block
    pub fn put_state_root(&self, block_hash: &Hash, state_root: &Hash) -> Result<()> {
        let key = state_root_key(block_hash);
        self.db.put_cf(CF_STATE, &key, state_root.as_bytes())?;
        Ok(())
    }

    /// Get state root for a block
    pub fn get_state_root(&self, block_hash: &Hash) -> Result<Option<Hash>> {
        let key = state_root_key(block_hash);
        match self.db.get_cf(CF_STATE, &key)? {
            Some(bytes) => Ok(Some(Hash::from_bytes(&bytes))),
            None => Ok(None),
        }
    }

    /// Create a state snapshot at a specific block
    pub fn create_snapshot(
        &self,
        block_hash: &Hash,
        accounts: Vec<(Address, AccountState)>,
    ) -> Result<()> {
        let mut batch = self.db.batch();

        for (address, account) in accounts {
            let snapshot_key = snapshot_account_key(block_hash, &address);
            let account_bytes = bincode::serialize(&account)?;
            self.db
                .batch_put_cf(&mut batch, CF_STATE, &snapshot_key, &account_bytes)?;
        }

        self.db.write_batch(batch)?;
        info!("Created state snapshot at block {}", block_hash);
        Ok(())
    }

    /// Get account from snapshot
    pub fn get_snapshot_account(
        &self,
        block_hash: &Hash,
        address: &Address,
    ) -> Result<Option<AccountState>> {
        let key = snapshot_account_key(block_hash, address);
        match self.db.get_cf(CF_STATE, &key)? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Compact state storage
    pub fn compact(&self) -> Result<()> {
        self.db.compact_cf(CF_STATE)?;
        self.db.compact_cf(CF_ACCOUNTS)?;
        self.db.compact_cf(CF_STORAGE)?;
        self.db.compact_cf(CF_CODE)?;
        self.db.compact_cf(CF_MODELS)?;
        self.db.compact_cf(CF_TRAINING)?;
        Ok(())
    }
}

// Key generation helpers
fn storage_key(address: &Address, key: &[u8]) -> Vec<u8> {
    let mut storage_key = Vec::with_capacity(20 + key.len());
    storage_key.extend_from_slice(&address.0);
    storage_key.extend_from_slice(key);
    storage_key
}

fn owner_model_key(owner: &Address, model_id: &ModelId) -> Vec<u8> {
    let mut key = vec![b'm'];
    key.extend_from_slice(&owner.0);
    key.extend_from_slice(model_id.0.as_bytes());
    key
}

fn owner_model_prefix(owner: &Address) -> Vec<u8> {
    let mut prefix = vec![b'm'];
    prefix.extend_from_slice(&owner.0);
    prefix
}

fn owner_job_key(owner: &Address, job_id: &JobId) -> Vec<u8> {
    let mut key = vec![b'j'];
    key.extend_from_slice(&owner.0);
    key.extend_from_slice(job_id.0.as_bytes());
    key
}

fn state_root_key(block_hash: &Hash) -> Vec<u8> {
    let mut key = vec![b'r'];
    key.extend_from_slice(block_hash.as_bytes());
    key
}

fn snapshot_account_key(block_hash: &Hash, address: &Address) -> Vec<u8> {
    let mut key = vec![b's'];
    key.extend_from_slice(block_hash.as_bytes());
    key.extend_from_slice(&address.0);
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_execution::types::Address;
    use primitive_types::U256;
    use tempfile::TempDir;

    #[test]
    fn test_account_storage() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(RocksDB::open(temp_dir.path()).unwrap());
        let store = StateStore::new(db);

        let address = Address([1; 20]);
        let account = AccountState {
            nonce: 10,
            balance: U256::from(1000),
            storage_root: Hash::default(),
            code_hash: Hash::default(),
            model_permissions: vec![],
        };

        // Store account
        store.put_account(&address, &account).unwrap();

        // Retrieve account
        let retrieved = store.get_account(&address).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().nonce, 10);
    }

    #[test]
    fn test_storage_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(RocksDB::open(temp_dir.path()).unwrap());
        let store = StateStore::new(db);

        let address = Address([1; 20]);

        // Store value
        store.put_storage(&address, b"key1", b"value1").unwrap();

        // Retrieve value
        let value = store.get_storage(&address, b"key1").unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // Delete value
        store.delete_storage(&address, b"key1").unwrap();
        assert!(store.get_storage(&address, b"key1").unwrap().is_none());
    }
}
