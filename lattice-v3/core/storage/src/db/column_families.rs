/// Column family definitions for RocksDB
pub const CF_DEFAULT: &str = "default";
pub const CF_BLOCKS: &str = "blocks";
pub const CF_HEADERS: &str = "headers";
pub const CF_TRANSACTIONS: &str = "transactions";
pub const CF_RECEIPTS: &str = "receipts";
pub const CF_STATE: &str = "state";
pub const CF_ACCOUNTS: &str = "accounts";
pub const CF_STORAGE: &str = "storage";
pub const CF_CODE: &str = "code";
pub const CF_MODELS: &str = "models";
pub const CF_TRAINING: &str = "training";
pub const CF_METADATA: &str = "metadata";
pub const CF_BLUE_SET: &str = "blue_set";
pub const CF_DAG_RELATIONS: &str = "dag_relations";

/// Get all column families
pub fn all_column_families() -> Vec<&'static str> {
    vec![
        CF_DEFAULT,
        CF_BLOCKS,
        CF_HEADERS,
        CF_TRANSACTIONS,
        CF_RECEIPTS,
        CF_STATE,
        CF_ACCOUNTS,
        CF_STORAGE,
        CF_CODE,
        CF_MODELS,
        CF_TRAINING,
        CF_METADATA,
        CF_BLUE_SET,
        CF_DAG_RELATIONS,
    ]
}