use rocksdb::{BlockBasedOptions, Cache, Options, SliceTransform};

/// RocksDB optimization configurations
pub struct DbOptimizations;

impl DbOptimizations {
    /// Create optimized options for the database
    pub fn optimized_db_options() -> Options {
        let mut opts = Options::default();

        // Basic optimizations
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        // Performance optimizations
        opts.set_max_background_jobs(4);
        opts.set_max_subcompactions(2);
        opts.set_bytes_per_sync(1048576); // 1MB
        opts.increase_parallelism(num_cpus::get() as i32);

        // Write optimizations
        opts.set_write_buffer_size(128 * 1024 * 1024); // 128MB
        opts.set_max_write_buffer_number(4);
        opts.set_min_write_buffer_number_to_merge(2);

        // Compaction optimizations
        opts.set_level_zero_file_num_compaction_trigger(4);
        opts.set_level_zero_slowdown_writes_trigger(20);
        opts.set_level_zero_stop_writes_trigger(30);

        // Enable pipelined writes for better throughput
        opts.set_enable_pipelined_write(true);

        // Use direct I/O for reads to bypass OS cache
        opts.set_use_direct_reads(true);
        opts.set_use_direct_io_for_flush_and_compaction(true);

        opts
    }

    /// Create block-based table options with bloom filters
    pub fn block_table_options(cache_size: usize) -> BlockBasedOptions {
        let mut block_opts = BlockBasedOptions::default();

        // Set block cache
        let cache = Cache::new_lru_cache(cache_size);
        block_opts.set_block_cache(&cache);

        // Enable bloom filter for point lookups
        block_opts.set_bloom_filter(10.0, false);

        // Set block size
        block_opts.set_block_size(16 * 1024); // 16KB blocks

        // Enable index and filter blocks cache
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);

        block_opts
    }

    /// Create prefix extractor for metadata keys
    pub fn metadata_prefix_extractor() -> SliceTransform {
        SliceTransform::create_fixed_prefix(1)
    }

    /// Optimize column family for metadata (headers, tips, etc.)
    pub fn optimize_metadata_cf(mut opts: Options) -> Options {
        // Metadata is accessed frequently but is relatively small
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB

        // Use bloom filter for faster lookups
        let mut block_opts = Self::block_table_options(256 * 1024 * 1024); // 256MB cache
        block_opts.set_bloom_filter(10.0, true); // Use block-based bloom filter
        opts.set_block_based_table_factory(&block_opts);

        // Set prefix extractor for efficient prefix scans
        opts.set_prefix_extractor(Self::metadata_prefix_extractor());

        // Optimize for point lookups
        opts.optimize_for_point_lookup(256); // 256MB block cache

        opts
    }

    /// Optimize column family for transactions
    pub fn optimize_transactions_cf(mut opts: Options) -> Options {
        // Transactions are write-heavy
        opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB
        opts.set_max_write_buffer_number(6);

        // Use Zstd compression for better compression ratio
        opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
        opts.set_bottommost_compression_type(rocksdb::DBCompressionType::Zstd);

        // Larger block size for sequential scans
        let mut block_opts = Self::block_table_options(512 * 1024 * 1024); // 512MB cache
        block_opts.set_block_size(64 * 1024); // 64KB blocks
        opts.set_block_based_table_factory(&block_opts);

        opts
    }

    /// Optimize column family for state data
    pub fn optimize_state_cf(mut opts: Options) -> Options {
        // State data has mixed read/write patterns
        opts.set_write_buffer_size(128 * 1024 * 1024); // 128MB

        // Use bloom filter for existence checks
        let mut block_opts = Self::block_table_options(1024 * 1024 * 1024); // 1GB cache
        block_opts.set_bloom_filter(10.0, false);
        block_opts.set_block_size(32 * 1024); // 32KB blocks
        opts.set_block_based_table_factory(&block_opts);

        // Enable prefix seeks for storage slots
        opts.set_prefix_extractor(SliceTransform::create_fixed_prefix(20)); // Address prefix

        opts
    }

    /// Optimize column family for receipts
    pub fn optimize_receipts_cf(mut opts: Options) -> Options {
        // Receipts are mostly write-once, read-occasionally
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB

        // Use higher compression for older data
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_bottommost_compression_type(rocksdb::DBCompressionType::Zstd);

        let block_opts = Self::block_table_options(128 * 1024 * 1024); // 128MB cache
        opts.set_block_based_table_factory(&block_opts);

        opts
    }

    /// Apply optimizations to all column families
    pub fn apply_cf_optimizations(cf_name: &str, opts: Options) -> Options {
        match cf_name {
            "headers" | "tips" | "mergeset" | "blue_score" => Self::optimize_metadata_cf(opts),
            "transactions" => Self::optimize_transactions_cf(opts),
            "state" | "accounts" | "storage" | "code" => Self::optimize_state_cf(opts),
            "receipts" | "logs" => Self::optimize_receipts_cf(opts),
            _ => opts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_options_creation() {
        let _opts = DbOptimizations::optimized_db_options();
        // Options should be created without panic. RocksDB Rust API lacks getters; compilation validates config build.
    }

    #[test]
    fn test_block_table_options() {
        let _block_opts = DbOptimizations::block_table_options(1024 * 1024);
        // Block options should be created successfully
        // Note: Can't directly test private fields, but creation shouldn't panic
    }

    #[test]
    fn test_cf_optimization() {
        let base_opts = Options::default();
        let _optimized = DbOptimizations::optimize_metadata_cf(base_opts);
        // Should return optimized options without panic
    }
}
