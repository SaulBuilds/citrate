// lattice-v3/core/marketplace/src/storage.rs

use crate::types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, error, info};

/// Marketplace storage using SQLite
pub struct MarketplaceStorage {
    pool: SqlitePool,
}

impl MarketplaceStorage {
    /// Create a new storage instance
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_url = format!("sqlite:{}", db_path.as_ref().display());

        // Ensure parent directory exists
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let pool = SqlitePool::connect(&db_url).await?;

        // Run migrations
        let storage = Self { pool };
        storage.migrate().await?;

        info!("Marketplace storage initialized at {}", db_url);
        Ok(storage)
    }

    /// Store a model
    pub async fn store_model(&self, model: &MarketplaceModel) -> Result<()> {
        let model_id_bytes = model.model_id.to_vec();
        let owner_bytes = model.owner.to_vec();
        let tags_json = serde_json::to_string(&model.tags)?;
        let input_shape_json = serde_json::to_string(&model.input_shape)?;
        let output_shape_json = serde_json::to_string(&model.output_shape)?;

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO models (
                model_id, owner, name, description, category,
                base_price, discount_price, minimum_bulk_size,
                framework, version, license, tags,
                input_shape, output_shape, parameters, size_bytes,
                model_cid, metadata_uri,
                total_sales, total_revenue, rating, review_count,
                featured, active, created_at, updated_at, last_sale_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8,
                ?9, ?10, ?11, ?12,
                ?13, ?14, ?15, ?16,
                ?17, ?18,
                ?19, ?20, ?21, ?22,
                ?23, ?24, ?25, ?26, ?27
            )
            "#,
            model_id_bytes,
            owner_bytes,
            model.name,
            model.description,
            model.category as u8,
            model.base_price as i64,
            model.discount_price as i64,
            model.minimum_bulk_size as i64,
            model.framework,
            model.version,
            model.license,
            tags_json,
            input_shape_json,
            output_shape_json,
            model.parameters as i64,
            model.size_bytes as i64,
            model.model_cid,
            model.metadata_uri,
            model.total_sales as i64,
            model.total_revenue as i64,
            model.rating,
            model.review_count as i64,
            model.featured,
            model.active,
            model.created_at.timestamp(),
            model.updated_at.timestamp(),
            model.last_sale_at.map(|dt| dt.timestamp())
        )
        .execute(&self.pool)
        .await?;

        debug!(model_id = ?model.model_id, "Model stored");
        Ok(())
    }

    /// Get a model by ID
    pub async fn get_model(&self, model_id: &ModelId) -> Result<Option<MarketplaceModel>> {
        let model_id_bytes = model_id.to_vec();

        let row = sqlx::query!(
            "SELECT * FROM models WHERE model_id = ?1",
            model_id_bytes
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_model(row)?))
        } else {
            Ok(None)
        }
    }

    /// Update a model
    pub async fn update_model(&self, model: &MarketplaceModel) -> Result<()> {
        self.store_model(model).await
    }

    /// Remove a model
    pub async fn remove_model(&self, model_id: &ModelId) -> Result<()> {
        let model_id_bytes = model_id.to_vec();

        sqlx::query!(
            "DELETE FROM models WHERE model_id = ?1",
            model_id_bytes
        )
        .execute(&self.pool)
        .await?;

        debug!(model_id = ?model_id, "Model removed");
        Ok(())
    }

    /// Get all models
    pub async fn get_all_models(&self) -> Result<Vec<MarketplaceModel>> {
        let rows = sqlx::query!("SELECT * FROM models ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut models = Vec::new();
        for row in rows {
            models.push(self.row_to_model(row)?);
        }

        Ok(models)
    }

    /// Get models by owner
    pub async fn get_models_by_owner(&self, owner: &Address) -> Result<Vec<MarketplaceModel>> {
        let owner_bytes = owner.to_vec();

        let rows = sqlx::query!(
            "SELECT * FROM models WHERE owner = ?1 ORDER BY created_at DESC",
            owner_bytes
        )
        .fetch_all(&self.pool)
        .await?;

        let mut models = Vec::new();
        for row in rows {
            models.push(self.row_to_model(row)?);
        }

        Ok(models)
    }

    /// Get model count
    pub async fn get_model_count(&self) -> Result<u64> {
        let row = sqlx::query!("SELECT COUNT(*) as count FROM models")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.count as u64)
    }

    /// Record user interaction
    pub async fn record_interaction(&self, interaction: &UserInteraction) -> Result<()> {
        let user_bytes = interaction.user.to_vec();
        let model_id_bytes = interaction.model_id.to_vec();
        let metadata_json = serde_json::to_string(&interaction.metadata)?;

        sqlx::query!(
            r#"
            INSERT INTO user_interactions (
                user_address, model_id, interaction_type, timestamp, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            user_bytes,
            model_id_bytes,
            interaction.interaction_type as u8,
            interaction.timestamp.timestamp(),
            metadata_json
        )
        .execute(&self.pool)
        .await?;

        debug!(
            user = ?interaction.user,
            model_id = ?interaction.model_id,
            interaction_type = ?interaction.interaction_type,
            "User interaction recorded"
        );
        Ok(())
    }

    /// Get user interactions
    pub async fn get_user_interactions(&self, user: &Address, limit: u32) -> Result<Vec<UserInteraction>> {
        let user_bytes = user.to_vec();

        let rows = sqlx::query!(
            r#"
            SELECT * FROM user_interactions
            WHERE user_address = ?1
            ORDER BY timestamp DESC
            LIMIT ?2
            "#,
            user_bytes,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let mut interactions = Vec::new();
        for row in rows {
            let mut user_addr = [0u8; 20];
            user_addr.copy_from_slice(&row.user_address);

            let mut model_id = [0u8; 32];
            model_id.copy_from_slice(&row.model_id);

            let interaction_type = match row.interaction_type {
                0 => InteractionType::View,
                1 => InteractionType::Purchase,
                2 => InteractionType::Review,
                3 => InteractionType::Bookmark,
                4 => InteractionType::Share,
                _ => continue,
            };

            let metadata: HashMap<String, String> = serde_json::from_str(&row.metadata)?;

            let timestamp = DateTime::from_timestamp(row.timestamp, 0)
                .unwrap_or_else(|| Utc::now());

            interactions.push(UserInteraction {
                user: user_addr,
                model_id,
                interaction_type,
                timestamp,
                metadata,
            });
        }

        Ok(interactions)
    }

    /// Store model review
    pub async fn store_review(&self, review: &ModelReview) -> Result<()> {
        let model_id_bytes = review.model_id.to_vec();
        let reviewer_bytes = review.reviewer.to_vec();

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO reviews (
                model_id, reviewer, rating, comment, verified, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            model_id_bytes,
            reviewer_bytes,
            review.rating,
            review.comment,
            review.verified,
            review.created_at.timestamp()
        )
        .execute(&self.pool)
        .await?;

        debug!(
            model_id = ?review.model_id,
            reviewer = ?review.reviewer,
            rating = review.rating,
            "Review stored"
        );
        Ok(())
    }

    /// Get reviews for a model
    pub async fn get_model_reviews(&self, model_id: &ModelId, limit: u32) -> Result<Vec<ModelReview>> {
        let model_id_bytes = model_id.to_vec();

        let rows = sqlx::query!(
            r#"
            SELECT * FROM reviews
            WHERE model_id = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
            model_id_bytes,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let mut reviews = Vec::new();
        for row in rows {
            let mut model_id = [0u8; 32];
            model_id.copy_from_slice(&row.model_id);

            let mut reviewer = [0u8; 20];
            reviewer.copy_from_slice(&row.reviewer);

            let created_at = DateTime::from_timestamp(row.created_at, 0)
                .unwrap_or_else(|| Utc::now());

            reviews.push(ModelReview {
                model_id,
                reviewer,
                rating: row.rating as u8,
                comment: row.comment,
                verified: row.verified,
                created_at,
            });
        }

        Ok(reviews)
    }

    /// Store purchase record
    pub async fn store_purchase(&self, purchase: &Purchase) -> Result<()> {
        let model_id_bytes = purchase.model_id.to_vec();
        let buyer_bytes = purchase.buyer.to_vec();

        sqlx::query!(
            r#"
            INSERT INTO purchases (
                model_id, buyer, price_per_inference, quantity,
                bulk_discount, transaction_hash, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            model_id_bytes,
            buyer_bytes,
            purchase.price_per_inference as i64,
            purchase.quantity as i64,
            purchase.bulk_discount,
            purchase.transaction_hash,
            purchase.timestamp.timestamp()
        )
        .execute(&self.pool)
        .await?;

        debug!(
            model_id = ?purchase.model_id,
            buyer = ?purchase.buyer,
            quantity = purchase.quantity,
            "Purchase recorded"
        );
        Ok(())
    }

    /// Get marketplace statistics
    pub async fn get_marketplace_stats(&self) -> Result<MarketplaceStats> {
        // Get total models
        let total_models = self.get_model_count().await?;

        // Get total sales and volume
        let sales_row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_sales,
                COALESCE(SUM(price_per_inference * quantity), 0) as total_volume
            FROM purchases
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let total_sales = sales_row.total_sales as u64;
        let total_volume = sales_row.total_volume as u64;

        // Get active users (users who have made interactions in the last 30 days)
        let active_users_row = sqlx::query!(
            r#"
            SELECT COUNT(DISTINCT user_address) as active_users
            FROM user_interactions
            WHERE timestamp > ?1
            "#,
            (Utc::now() - chrono::Duration::days(30)).timestamp()
        )
        .fetch_one(&self.pool)
        .await?;

        let active_users = active_users_row.active_users as u64;

        // Get category counts
        let category_rows = sqlx::query!(
            r#"
            SELECT category, COUNT(*) as count
            FROM models
            WHERE active = 1
            GROUP BY category
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut categories_count = HashMap::new();
        for row in category_rows {
            let category = ModelCategory::from(row.category as u8);
            categories_count.insert(category, row.count as u64);
        }

        // Get top models by sales
        let top_model_rows = sqlx::query!(
            r#"
            SELECT model_id, SUM(quantity) as total_quantity
            FROM purchases p
            JOIN models m ON p.model_id = m.model_id
            WHERE m.active = 1
            GROUP BY model_id
            ORDER BY total_quantity DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut top_models = Vec::new();
        for row in top_model_rows {
            let mut model_id = [0u8; 32];
            model_id.copy_from_slice(&row.model_id);
            top_models.push(model_id);
        }

        // Get trending models (models with recent sales activity)
        let trending_rows = sqlx::query!(
            r#"
            SELECT model_id, COUNT(*) as recent_sales
            FROM purchases p
            JOIN models m ON p.model_id = m.model_id
            WHERE m.active = 1 AND p.timestamp > ?1
            GROUP BY model_id
            ORDER BY recent_sales DESC
            LIMIT 10
            "#,
            (Utc::now() - chrono::Duration::days(7)).timestamp()
        )
        .fetch_all(&self.pool)
        .await?;

        let mut trending_models = Vec::new();
        for row in trending_rows {
            let mut model_id = [0u8; 32];
            model_id.copy_from_slice(&row.model_id);
            trending_models.push(model_id);
        }

        Ok(MarketplaceStats {
            total_models,
            total_sales,
            total_volume,
            active_users,
            categories_count,
            top_models,
            trending_models,
            updated_at: Utc::now(),
        })
    }

    /// Flush any pending changes
    pub async fn flush(&self) -> Result<()> {
        // SQLite auto-commits, so nothing to do here
        Ok(())
    }

    // Private methods

    async fn migrate(&self) -> Result<()> {
        // Create models table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS models (
                model_id BLOB PRIMARY KEY,
                owner BLOB NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                category INTEGER NOT NULL,
                base_price INTEGER NOT NULL,
                discount_price INTEGER NOT NULL,
                minimum_bulk_size INTEGER NOT NULL,
                framework TEXT NOT NULL,
                version TEXT NOT NULL,
                license TEXT NOT NULL,
                tags TEXT NOT NULL,
                input_shape TEXT NOT NULL,
                output_shape TEXT NOT NULL,
                parameters INTEGER NOT NULL,
                size_bytes INTEGER NOT NULL,
                model_cid TEXT NOT NULL,
                metadata_uri TEXT NOT NULL,
                total_sales INTEGER NOT NULL DEFAULT 0,
                total_revenue INTEGER NOT NULL DEFAULT 0,
                rating REAL NOT NULL DEFAULT 0.0,
                review_count INTEGER NOT NULL DEFAULT 0,
                featured BOOLEAN NOT NULL DEFAULT 0,
                active BOOLEAN NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                last_sale_at INTEGER
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create user_interactions table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS user_interactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_address BLOB NOT NULL,
                model_id BLOB NOT NULL,
                interaction_type INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}'
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create reviews table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS reviews (
                model_id BLOB NOT NULL,
                reviewer BLOB NOT NULL,
                rating INTEGER NOT NULL,
                comment TEXT NOT NULL,
                verified BOOLEAN NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (model_id, reviewer)
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create purchases table
        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS purchases (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                model_id BLOB NOT NULL,
                buyer BLOB NOT NULL,
                price_per_inference INTEGER NOT NULL,
                quantity INTEGER NOT NULL,
                bulk_discount BOOLEAN NOT NULL DEFAULT 0,
                transaction_hash TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_models_owner ON models(owner)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_models_category ON models(category)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_models_featured ON models(featured)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_models_active ON models(active)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_interactions_user ON user_interactions(user_address)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_interactions_model ON user_interactions(model_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_interactions_timestamp ON user_interactions(timestamp)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_reviews_model ON reviews(model_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_purchases_model ON purchases(model_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_purchases_buyer ON purchases(buyer)")
            .execute(&self.pool)
            .await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_purchases_timestamp ON purchases(timestamp)")
            .execute(&self.pool)
            .await?;

        info!("Database migration completed");
        Ok(())
    }

    fn row_to_model(&self, row: sqlx::sqlite::SqliteRow) -> Result<MarketplaceModel> {
        let model_id_bytes: Vec<u8> = row.get("model_id");
        let owner_bytes: Vec<u8> = row.get("owner");

        let mut model_id = [0u8; 32];
        let mut owner = [0u8; 20];

        model_id.copy_from_slice(&model_id_bytes);
        owner.copy_from_slice(&owner_bytes);

        let tags_json: String = row.get("tags");
        let input_shape_json: String = row.get("input_shape");
        let output_shape_json: String = row.get("output_shape");

        let tags: Vec<String> = serde_json::from_str(&tags_json)?;
        let input_shape: Vec<String> = serde_json::from_str(&input_shape_json)?;
        let output_shape: Vec<String> = serde_json::from_str(&output_shape_json)?;

        let created_at_ts: i64 = row.get("created_at");
        let updated_at_ts: i64 = row.get("updated_at");
        let last_sale_at_ts: Option<i64> = row.get("last_sale_at");

        let created_at = DateTime::from_timestamp(created_at_ts, 0)
            .unwrap_or_else(|| Utc::now());
        let updated_at = DateTime::from_timestamp(updated_at_ts, 0)
            .unwrap_or_else(|| Utc::now());
        let last_sale_at = last_sale_at_ts
            .and_then(|ts| DateTime::from_timestamp(ts, 0));

        let category_u8: u8 = row.get::<i64, _>("category") as u8;
        let category = ModelCategory::from(category_u8);

        Ok(MarketplaceModel {
            model_id,
            owner,
            name: row.get("name"),
            description: row.get("description"),
            category,
            base_price: row.get::<i64, _>("base_price") as u64,
            discount_price: row.get::<i64, _>("discount_price") as u64,
            minimum_bulk_size: row.get::<i64, _>("minimum_bulk_size") as u32,
            framework: row.get("framework"),
            version: row.get("version"),
            license: row.get("license"),
            tags,
            input_shape,
            output_shape,
            parameters: row.get::<i64, _>("parameters") as u64,
            size_bytes: row.get::<i64, _>("size_bytes") as u64,
            model_cid: row.get("model_cid"),
            metadata_uri: row.get("metadata_uri"),
            total_sales: row.get::<i64, _>("total_sales") as u64,
            total_revenue: row.get::<i64, _>("total_revenue") as u64,
            rating: row.get("rating"),
            review_count: row.get::<i64, _>("review_count") as u32,
            featured: row.get("featured"),
            active: row.get("active"),
            created_at,
            updated_at,
            last_sale_at,
        })
    }
}