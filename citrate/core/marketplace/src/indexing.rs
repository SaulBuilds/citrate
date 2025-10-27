// citrate/core/marketplace/src/indexing.rs

use crate::{
    metadata::MetadataCache,
    search::SearchEngine,
    storage::MarketplaceStorage,
    types::*,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Batch size for processing models
const BATCH_SIZE: usize = 100;

/// Indexing operations
#[derive(Debug, Clone)]
pub enum IndexingOperation {
    AddModel(MarketplaceModel),
    UpdateModel(MarketplaceModel),
    RemoveModel(ModelId),
    ReindexAll,
    OptimizeIndex,
}

/// Indexing task with priority
#[derive(Debug, Clone)]
pub struct IndexingTask {
    pub operation: IndexingOperation,
    pub priority: u8, // 0 = highest, 255 = lowest
    pub retry_count: u8,
}

/// Statistics for the indexing service
#[derive(Debug, Default, Clone)]
pub struct IndexingStats {
    pub models_indexed: u64,
    pub models_updated: u64,
    pub models_removed: u64,
    pub failed_operations: u64,
    pub queue_size: usize,
    pub last_full_reindex: Option<chrono::DateTime<chrono::Utc>>,
    pub last_optimization: Option<chrono::DateTime<chrono::Utc>>,
}

/// Background indexing service that manages the search index
pub struct IndexingService {
    search_engine: Arc<SearchEngine>,
    storage: Arc<MarketplaceStorage>,
    metadata_cache: Arc<MetadataCache>,
    task_queue: Arc<RwLock<Vec<IndexingTask>>>,
    stats: Arc<RwLock<IndexingStats>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl IndexingService {
    /// Create a new indexing service
    pub fn new(
        search_engine: Arc<SearchEngine>,
        storage: Arc<MarketplaceStorage>,
        metadata_cache: Arc<MetadataCache>,
    ) -> Self {
        Self {
            search_engine,
            storage,
            metadata_cache,
            task_queue: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(IndexingStats::default())),
            shutdown_tx: None,
        }
    }

    /// Start the background indexing service
    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let search_engine = Arc::clone(&self.search_engine);
        let storage = Arc::clone(&self.storage);
        let metadata_cache = Arc::clone(&self.metadata_cache);
        let task_queue = Arc::clone(&self.task_queue);
        let stats = Arc::clone(&self.stats);

        // Start the main processing loop
        tokio::spawn(async move {
            let mut processing_interval = interval(Duration::from_millis(100));
            let mut optimization_interval = interval(Duration::from_secs(3600)); // Every hour
            let mut full_reindex_interval = interval(Duration::from_secs(86400)); // Daily

            info!("Indexing service started");

            loop {
                tokio::select! {
                    _ = processing_interval.tick() => {
                        if let Err(e) = Self::process_queue(
                            Arc::clone(&search_engine),
                            Arc::clone(&storage),
                            Arc::clone(&metadata_cache),
                            Arc::clone(&task_queue),
                            Arc::clone(&stats),
                        ).await {
                            error!(error = %e, "Error processing indexing queue");
                        }
                    }
                    _ = optimization_interval.tick() => {
                        if let Err(e) = Self::optimize_index(Arc::clone(&search_engine), Arc::clone(&stats)).await {
                            error!(error = %e, "Error optimizing search index");
                        }
                    }
                    _ = full_reindex_interval.tick() => {
                        if let Err(e) = Self::schedule_full_reindex(Arc::clone(&task_queue)).await {
                            error!(error = %e, "Error scheduling full reindex");
                        }
                    }
                    _ = &mut shutdown_rx => {
                        info!("Indexing service shutdown requested");
                        break;
                    }
                }
            }

            info!("Indexing service stopped");
        });

        Ok(())
    }

    /// Stop the indexing service
    pub async fn stop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
    }

    /// Add a model to the index
    pub async fn index_model(&self, model: MarketplaceModel) -> Result<()> {
        let task = IndexingTask {
            operation: IndexingOperation::AddModel(model),
            priority: 1, // High priority for new models
            retry_count: 0,
        };

        self.enqueue_task(task).await;
        Ok(())
    }

    /// Update a model in the index
    pub async fn update_model(&self, model: MarketplaceModel) -> Result<()> {
        let task = IndexingTask {
            operation: IndexingOperation::UpdateModel(model),
            priority: 2, // Medium priority for updates
            retry_count: 0,
        };

        self.enqueue_task(task).await;
        Ok(())
    }

    /// Remove a model from the index
    pub async fn remove_model(&self, model_id: ModelId) -> Result<()> {
        let task = IndexingTask {
            operation: IndexingOperation::RemoveModel(model_id),
            priority: 1, // High priority for removals
            retry_count: 0,
        };

        self.enqueue_task(task).await;
        Ok(())
    }

    /// Schedule a full reindex of all models
    pub async fn reindex_all(&self) -> Result<()> {
        let task = IndexingTask {
            operation: IndexingOperation::ReindexAll,
            priority: 10, // Lower priority for full reindex
            retry_count: 0,
        };

        self.enqueue_task(task).await;
        Ok(())
    }

    /// Get indexing statistics
    pub async fn get_stats(&self) -> IndexingStats {
        let stats = self.stats.read().await;
        let queue_size = self.task_queue.read().await.len();

        IndexingStats {
            queue_size,
            ..stats.clone()
        }
    }

    /// Get queue size
    pub async fn get_queue_size(&self) -> usize {
        self.task_queue.read().await.len()
    }

    /// Clear the indexing queue
    pub async fn clear_queue(&self) {
        let mut queue = self.task_queue.write().await;
        queue.clear();
        info!("Indexing queue cleared");
    }

    // Private methods

    async fn enqueue_task(&self, task: IndexingTask) {
        let mut queue = self.task_queue.write().await;

        debug!(
            operation = ?task.operation,
            priority = task.priority,
            "Task enqueued"
        );

        queue.push(task);

        // Sort by priority (lower number = higher priority)
        queue.sort_by_key(|task| task.priority);

        debug!(queue_size = queue.len(), "Queue updated");
    }

    async fn process_queue(
        search_engine: Arc<SearchEngine>,
        storage: Arc<MarketplaceStorage>,
        metadata_cache: Arc<MetadataCache>,
        task_queue: Arc<RwLock<Vec<IndexingTask>>>,
        stats: Arc<RwLock<IndexingStats>>,
    ) -> Result<()> {
        let tasks = {
            let mut queue = task_queue.write().await;
            if queue.is_empty() {
                return Ok(());
            }

            // Take up to BATCH_SIZE tasks
            let batch_size = queue.len().min(BATCH_SIZE);
            queue.drain(0..batch_size).collect::<Vec<_>>()
        };

        if tasks.is_empty() {
            return Ok(());
        }

        debug!(task_count = tasks.len(), "Processing indexing batch");

        for mut task in tasks {
            let result = Self::process_task(
                &task,
                Arc::clone(&search_engine),
                Arc::clone(&storage),
                Arc::clone(&metadata_cache),
            ).await;

            match result {
                Ok(()) => {
                    Self::update_stats_for_success(&task, Arc::clone(&stats)).await;
                    debug!(operation = ?task.operation, "Task completed successfully");
                }
                Err(e) => {
                    error!(
                        operation = ?task.operation,
                        error = %e,
                        retry_count = task.retry_count,
                        "Task failed"
                    );

                    // Retry logic
                    task.retry_count += 1;
                    if task.retry_count < 3 {
                        // Re-enqueue with lower priority
                        task.priority = task.priority.saturating_add(5);
                        let mut queue = task_queue.write().await;
                        queue.push(task);
                        queue.sort_by_key(|task| task.priority);
                    } else {
                        warn!(operation = ?task.operation, "Task failed after max retries");
                        let mut stats = stats.write().await;
                        stats.failed_operations += 1;
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_task(
        task: &IndexingTask,
        search_engine: Arc<SearchEngine>,
        storage: Arc<MarketplaceStorage>,
        metadata_cache: Arc<MetadataCache>,
    ) -> Result<()> {
        match &task.operation {
            IndexingOperation::AddModel(model) => {
                // Prefetch metadata if available
                if !model.metadata_uri.is_empty() {
                    if let Err(e) = metadata_cache.get_metadata(&model.metadata_uri).await {
                        warn!(
                            model_id = ?model.model_id,
                            metadata_uri = %model.metadata_uri,
                            error = %e,
                            "Failed to prefetch metadata during indexing"
                        );
                    }
                }

                search_engine.index_model(model).await?;
                storage.store_model(model).await?;
            }

            IndexingOperation::UpdateModel(model) => {
                search_engine.index_model(model).await?;
                storage.update_model(model).await?;
            }

            IndexingOperation::RemoveModel(model_id) => {
                search_engine.remove_model(model_id).await?;
                storage.remove_model(model_id).await?;
            }

            IndexingOperation::ReindexAll => {
                Self::perform_full_reindex(search_engine, storage).await?;
            }

            IndexingOperation::OptimizeIndex => {
                search_engine.optimize().await?;
            }
        }

        Ok(())
    }

    async fn perform_full_reindex(
        search_engine: Arc<SearchEngine>,
        storage: Arc<MarketplaceStorage>,
    ) -> Result<()> {
        info!("Starting full reindex of all models");

        let models = storage.get_all_models().await?;
        let mut indexed_count = 0;
        let mut failed_count = 0;

        for model in models {
            match search_engine.index_model(&model).await {
                Ok(()) => indexed_count += 1,
                Err(e) => {
                    error!(
                        model_id = ?model.model_id,
                        error = %e,
                        "Failed to index model during full reindex"
                    );
                    failed_count += 1;
                }
            }

            // Yield periodically to prevent blocking
            if indexed_count % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }

        search_engine.commit().await?;

        info!(
            indexed_count = indexed_count,
            failed_count = failed_count,
            "Full reindex completed"
        );

        Ok(())
    }

    async fn optimize_index(
        search_engine: Arc<SearchEngine>,
        stats: Arc<RwLock<IndexingStats>>,
    ) -> Result<()> {
        debug!("Optimizing search index");
        search_engine.optimize().await?;

        let mut stats = stats.write().await;
        stats.last_optimization = Some(chrono::Utc::now());

        info!("Search index optimization completed");
        Ok(())
    }

    async fn schedule_full_reindex(
        task_queue: Arc<RwLock<Vec<IndexingTask>>>,
    ) -> Result<()> {
        let task = IndexingTask {
            operation: IndexingOperation::ReindexAll,
            priority: 20, // Low priority for scheduled reindex
            retry_count: 0,
        };

        let mut queue = task_queue.write().await;

        // Check if full reindex is already queued
        let has_reindex = queue.iter().any(|t| {
            matches!(t.operation, IndexingOperation::ReindexAll)
        });

        if !has_reindex {
            queue.push(task);
            queue.sort_by_key(|task| task.priority);
            debug!("Scheduled automatic full reindex");
        }

        Ok(())
    }

    async fn update_stats_for_success(
        task: &IndexingTask,
        stats: Arc<RwLock<IndexingStats>>,
    ) {
        let mut stats = stats.write().await;

        match &task.operation {
            IndexingOperation::AddModel(_) => stats.models_indexed += 1,
            IndexingOperation::UpdateModel(_) => stats.models_updated += 1,
            IndexingOperation::RemoveModel(_) => stats.models_removed += 1,
            IndexingOperation::ReindexAll => {
                stats.last_full_reindex = Some(chrono::Utc::now());
            }
            IndexingOperation::OptimizeIndex => {
                stats.last_optimization = Some(chrono::Utc::now());
            }
        }
    }
}

/// Batch indexing operations for efficiency
pub struct BatchIndexer {
    service: Arc<IndexingService>,
    pending_adds: Vec<MarketplaceModel>,
    pending_updates: Vec<MarketplaceModel>,
    pending_removes: Vec<ModelId>,
    batch_size: usize,
}

impl BatchIndexer {
    pub fn new(service: Arc<IndexingService>) -> Self {
        Self {
            service,
            pending_adds: Vec::new(),
            pending_updates: Vec::new(),
            pending_removes: Vec::new(),
            batch_size: BATCH_SIZE,
        }
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Add model to batch
    pub fn add_model(&mut self, model: MarketplaceModel) {
        self.pending_adds.push(model);
    }

    /// Update model in batch
    pub fn update_model(&mut self, model: MarketplaceModel) {
        self.pending_updates.push(model);
    }

    /// Remove model in batch
    pub fn remove_model(&mut self, model_id: ModelId) {
        self.pending_removes.push(model_id);
    }

    /// Flush all pending operations
    pub async fn flush(&mut self) -> Result<()> {
        // Process additions
        for model in self.pending_adds.drain(..) {
            self.service.index_model(model).await?;
        }

        // Process updates
        for model in self.pending_updates.drain(..) {
            self.service.update_model(model).await?;
        }

        // Process removals
        for model_id in self.pending_removes.drain(..) {
            self.service.remove_model(model_id).await?;
        }

        Ok(())
    }

    /// Auto-flush when batch size is reached
    pub async fn maybe_flush(&mut self) -> Result<()> {
        let total_pending = self.pending_adds.len()
            + self.pending_updates.len()
            + self.pending_removes.len();

        if total_pending >= self.batch_size {
            self.flush().await?;
        }

        Ok(())
    }
}

impl Drop for BatchIndexer {
    fn drop(&mut self) {
        if !self.pending_adds.is_empty()
            || !self.pending_updates.is_empty()
            || !self.pending_removes.is_empty() {
            warn!("BatchIndexer dropped with pending operations - data may be lost");
        }
    }
}