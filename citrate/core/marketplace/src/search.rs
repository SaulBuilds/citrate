// citrate/core/marketplace/src/search.rs

use crate::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::{BooleanQuery, QueryParser, TermQuery},
    schema::{Field, Schema, STORED, STRING, TEXT, FAST, U64},
    Index, IndexReader, IndexWriter, ReloadPolicy, Score, Searcher, Term,
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Search query structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub filters: SearchFilters,
    pub sort_by: SortBy,
    pub limit: usize,
    pub offset: usize,
}

/// Search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub model: MarketplaceModel,
    pub score: f32,
    pub highlights: Vec<String>,
}

/// Full-text search engine using Tantivy
pub struct SearchEngine {
    index: Index,
    reader: IndexReader,
    writer: RwLock<IndexWriter>,
    query_parser: QueryParser,
    schema: Schema,

    // Schema fields
    model_id_field: Field,
    name_field: Field,
    description_field: Field,
    tags_field: Field,
    category_field: Field,
    framework_field: Field,
    license_field: Field,
    owner_field: Field,
    price_field: Field,
    rating_field: Field,
    sales_field: Field,
    created_at_field: Field,
    featured_field: Field,
    active_field: Field,
    full_text_field: Field,
}

impl SearchEngine {
    /// Create a new search engine with the given index directory
    pub async fn new<P: AsRef<Path>>(index_dir: P) -> Result<Self> {
        let index_path = index_dir.as_ref();

        // Create schema
        let mut schema_builder = Schema::builder();

        let model_id_field = schema_builder.add_bytes_field("model_id", STORED | FAST);
        let name_field = schema_builder.add_text_field("name", TEXT | STORED);
        let description_field = schema_builder.add_text_field("description", TEXT | STORED);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);
        let category_field = schema_builder.add_u64_field("category", STORED | FAST);
        let framework_field = schema_builder.add_text_field("framework", STRING | STORED | FAST);
        let license_field = schema_builder.add_text_field("license", STRING | STORED | FAST);
        let owner_field = schema_builder.add_bytes_field("owner", STORED | FAST);
        let price_field = schema_builder.add_u64_field("price", STORED | FAST);
        let rating_field = schema_builder.add_u64_field("rating_x100", STORED | FAST); // Store as rating * 100
        let sales_field = schema_builder.add_u64_field("sales", STORED | FAST);
        let created_at_field = schema_builder.add_u64_field("created_at", STORED | FAST);
        let featured_field = schema_builder.add_u64_field("featured", STORED | FAST);
        let active_field = schema_builder.add_u64_field("active", STORED | FAST);

        // Combined full-text field for general search
        let full_text_field = schema_builder.add_text_field("full_text", TEXT);

        let schema = schema_builder.build();

        // Create or open index
        let index = if index_path.exists() {
            Index::open(MmapDirectory::open(index_path)?)?
        } else {
            std::fs::create_dir_all(index_path)?;
            let dir = MmapDirectory::open(index_path)?;
            Index::create(dir, schema.clone())?
        };

        // Create reader and writer
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let writer = index.writer(50_000_000)?; // 50MB buffer

        // Create query parser for full-text search
        let query_parser = QueryParser::for_index(
            &index,
            vec![name_field, description_field, tags_field, full_text_field],
        );

        Ok(SearchEngine {
            index,
            reader,
            writer: RwLock::new(writer),
            query_parser,
            schema,
            model_id_field,
            name_field,
            description_field,
            tags_field,
            category_field,
            framework_field,
            license_field,
            owner_field,
            price_field,
            rating_field,
            sales_field,
            created_at_field,
            featured_field,
            active_field,
            full_text_field,
        })
    }

    /// Index a model for search
    pub async fn index_model(&self, model: &MarketplaceModel) -> Result<()> {
        let mut writer = self.writer.write().await;

        // Create full-text content for search
        let full_text = format!(
            "{} {} {} {}",
            model.name,
            model.description,
            model.tags.join(" "),
            model.framework
        );

        let doc = doc!(
            self.model_id_field => model.model_id.to_vec(),
            self.name_field => model.name.clone(),
            self.description_field => model.description.clone(),
            self.tags_field => model.tags.join(" "),
            self.category_field => model.category as u8 as u64,
            self.framework_field => model.framework.clone(),
            self.license_field => model.license.clone(),
            self.owner_field => model.owner.to_vec(),
            self.price_field => model.base_price,
            self.rating_field => (model.rating * 100.0) as u64,
            self.sales_field => model.total_sales,
            self.created_at_field => model.created_at.timestamp() as u64,
            self.featured_field => if model.featured { 1u64 } else { 0u64 },
            self.active_field => if model.active { 1u64 } else { 0u64 },
            self.full_text_field => full_text,
        );

        writer.add_document(doc)?;

        debug!(model_id = ?model.model_id, "Indexed model for search");
        Ok(())
    }

    /// Remove a model from the search index
    pub async fn remove_model(&self, model_id: &ModelId) -> Result<()> {
        let mut writer = self.writer.write().await;

        let term = Term::from_field_bytes(self.model_id_field, model_id);
        writer.delete_term(term);

        debug!(model_id = ?model_id, "Removed model from search index");
        Ok(())
    }

    /// Commit all pending changes to the index
    pub async fn commit(&self) -> Result<()> {
        let mut writer = self.writer.write().await;
        writer.commit()?;
        self.reader.reload()?;

        info!("Committed changes to search index");
        Ok(())
    }

    /// Search for models with the given query
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();

        // Build the search query
        let tantivy_query = self.build_tantivy_query(query)?;

        // Determine limit with offset
        let total_limit = query.limit + query.offset;

        // Execute search
        let top_docs = searcher.search(&tantivy_query, &TopDocs::with_limit(total_limit))?;

        // Convert results
        let mut results = Vec::new();
        for (score, doc_address) in top_docs.into_iter().skip(query.offset) {
            if let Ok(doc) = searcher.doc(doc_address) {
                if let Some(model) = self.doc_to_model(&doc)? {
                    results.push(SearchResult {
                        model,
                        score,
                        highlights: Vec::new(), // TODO: Implement highlighting
                    });
                }
            }
        }

        // Apply sorting if not relevance
        if !matches!(query.sort_by, SortBy::Relevance) {
            self.sort_results(&mut results, query.sort_by);
        }

        debug!(
            query = ?query.text,
            filters = ?query.filters,
            results_count = results.len(),
            "Executed search query"
        );

        Ok(results)
    }

    /// Get trending models based on recent activity
    pub async fn get_trending_models(&self, limit: usize) -> Result<Vec<MarketplaceModel>> {
        let searcher = self.reader.searcher();

        // Search for active models sorted by recent sales activity
        let query = tantivy::query::AllQuery;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit * 2))?;

        let mut models = Vec::new();
        for (_score, doc_address) in top_docs {
            if let Ok(doc) = searcher.doc(doc_address) {
                if let Some(model) = self.doc_to_model(&doc)? {
                    if model.active {
                        models.push(model);
                    }
                }
            }
        }

        // Sort by a combination of recent sales and rating
        models.sort_by(|a, b| {
            let score_a = (a.total_sales as f32 * 0.7) + (a.rating * 0.3);
            let score_b = (b.total_sales as f32 * 0.7) + (b.rating * 0.3);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        models.truncate(limit);
        Ok(models)
    }

    /// Get model suggestions based on a specific model
    pub async fn get_similar_models(&self, model_id: &ModelId, limit: usize) -> Result<Vec<MarketplaceModel>> {
        // First, find the target model
        let searcher = self.reader.searcher();
        let term = Term::from_field_bytes(self.model_id_field, model_id);
        let term_query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);

        let target_docs = searcher.search(&term_query, &TopDocs::with_limit(1))?;

        if target_docs.is_empty() {
            warn!(model_id = ?model_id, "Target model not found for similarity search");
            return Ok(Vec::new());
        }

        let target_doc = searcher.doc(target_docs[0].1)?;
        let target_model = self.doc_to_model(&target_doc)?
            .ok_or_else(|| anyhow::anyhow!("Failed to parse target model"))?;

        // Search for models in the same category
        let category_term = Term::from_field_u64(self.category_field, target_model.category as u8 as u64);
        let category_query = TermQuery::new(category_term, tantivy::schema::IndexRecordOption::Basic);

        let similar_docs = searcher.search(&category_query, &TopDocs::with_limit(limit * 2))?;

        let mut similar_models = Vec::new();
        for (_score, doc_address) in similar_docs {
            if let Ok(doc) = searcher.doc(doc_address) {
                if let Some(model) = self.doc_to_model(&doc)? {
                    if model.model_id != *model_id && model.active {
                        similar_models.push(model);
                    }
                }
            }
        }

        // Sort by rating and sales
        similar_models.sort_by(|a, b| {
            let score_a = (a.rating * 0.6) + (a.total_sales as f32 * 0.4);
            let score_b = (b.rating * 0.6) + (b.total_sales as f32 * 0.4);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        similar_models.truncate(limit);
        Ok(similar_models)
    }

    /// Get search statistics
    pub async fn get_stats(&self) -> Result<(usize, usize)> {
        let searcher = self.reader.searcher();
        let num_docs = searcher.num_docs() as usize;
        let num_segments = searcher.segment_readers().len();

        Ok((num_docs, num_segments))
    }

    // Private helper methods

    fn build_tantivy_query(&self, query: &SearchQuery) -> Result<Box<dyn tantivy::query::Query>> {
        use tantivy::query::{BooleanQuery, Occur, Query, RangeQuery};

        let mut subqueries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        // Text query
        if let Some(text) = &query.text {
            if !text.trim().is_empty() {
                let text_query = self.query_parser.parse_query(text)?;
                subqueries.push((Occur::Must, text_query));
            }
        }

        // Active filter (always applied)
        let active_term = Term::from_field_u64(self.active_field, 1u64);
        let active_query = TermQuery::new(active_term, tantivy::schema::IndexRecordOption::Basic);
        subqueries.push((Occur::Must, Box::new(active_query)));

        // Category filters
        if let Some(categories) = &query.filters.categories {
            if !categories.is_empty() {
                let mut category_queries = Vec::new();
                for category in categories {
                    let term = Term::from_field_u64(self.category_field, *category as u8 as u64);
                    let term_query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
                    category_queries.push((Occur::Should, Box::new(term_query) as Box<dyn Query>));
                }
                let category_bool_query = BooleanQuery::new(category_queries);
                subqueries.push((Occur::Must, Box::new(category_bool_query)));
            }
        }

        // Price range filter
        if query.filters.min_price.is_some() || query.filters.max_price.is_some() {
            let min_price = query.filters.min_price.unwrap_or(0);
            let max_price = query.filters.max_price.unwrap_or(u64::MAX);

            let price_range = RangeQuery::new_u64_bounds(
                self.price_field,
                std::ops::Bound::Included(min_price),
                std::ops::Bound::Included(max_price),
            );
            subqueries.push((Occur::Must, Box::new(price_range)));
        }

        // Rating filter
        if let Some(min_rating) = query.filters.min_rating {
            let min_rating_scaled = (min_rating * 100.0) as u64;
            let rating_range = RangeQuery::new_u64_bounds(
                self.rating_field,
                std::ops::Bound::Included(min_rating_scaled),
                std::ops::Bound::Unbounded,
            );
            subqueries.push((Occur::Must, Box::new(rating_range)));
        }

        // Featured filter
        if query.filters.featured_only {
            let featured_term = Term::from_field_u64(self.featured_field, 1u64);
            let featured_query = TermQuery::new(featured_term, tantivy::schema::IndexRecordOption::Basic);
            subqueries.push((Occur::Must, Box::new(featured_query)));
        }

        // Framework filters
        if let Some(frameworks) = &query.filters.frameworks {
            if !frameworks.is_empty() {
                let mut framework_queries = Vec::new();
                for framework in frameworks {
                    let term = Term::from_field_text(self.framework_field, framework);
                    let term_query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
                    framework_queries.push((Occur::Should, Box::new(term_query) as Box<dyn Query>));
                }
                let framework_bool_query = BooleanQuery::new(framework_queries);
                subqueries.push((Occur::Must, Box::new(framework_bool_query)));
            }
        }

        // License filters
        if let Some(licenses) = &query.filters.licenses {
            if !licenses.is_empty() {
                let mut license_queries = Vec::new();
                for license in licenses {
                    let term = Term::from_field_text(self.license_field, license);
                    let term_query = TermQuery::new(term, tantivy::schema::IndexRecordOption::Basic);
                    license_queries.push((Occur::Should, Box::new(term_query) as Box<dyn Query>));
                }
                let license_bool_query = BooleanQuery::new(license_queries);
                subqueries.push((Occur::Must, Box::new(license_bool_query)));
            }
        }

        if subqueries.is_empty() {
            // If no specific queries, return all active models
            let active_term = Term::from_field_u64(self.active_field, 1u64);
            let active_query = TermQuery::new(active_term, tantivy::schema::IndexRecordOption::Basic);
            Ok(Box::new(active_query))
        } else {
            Ok(Box::new(BooleanQuery::new(subqueries)))
        }
    }

    fn doc_to_model(&self, doc: &tantivy::Document) -> Result<Option<MarketplaceModel>> {
        // Extract model_id
        let model_id_bytes = doc
            .get_first(self.model_id_field)
            .and_then(|v| v.as_bytes())
            .ok_or_else(|| anyhow::anyhow!("Missing model_id field"))?;

        if model_id_bytes.len() != 32 {
            return Ok(None);
        }

        let mut model_id = [0u8; 32];
        model_id.copy_from_slice(model_id_bytes);

        // Extract owner
        let owner_bytes = doc
            .get_first(self.owner_field)
            .and_then(|v| v.as_bytes())
            .ok_or_else(|| anyhow::anyhow!("Missing owner field"))?;

        if owner_bytes.len() != 20 {
            return Ok(None);
        }

        let mut owner = [0u8; 20];
        owner.copy_from_slice(owner_bytes);

        // Extract other fields
        let name = doc
            .get_first(self.name_field)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();

        let description = doc
            .get_first(self.description_field)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();

        let category_u8 = doc
            .get_first(self.category_field)
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as u8;

        let category = ModelCategory::from(category_u8);

        let framework = doc
            .get_first(self.framework_field)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();

        let license = doc
            .get_first(self.license_field)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();

        let base_price = doc
            .get_first(self.price_field)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let rating_scaled = doc
            .get_first(self.rating_field)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let rating = (rating_scaled as f32) / 100.0;

        let total_sales = doc
            .get_first(self.sales_field)
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let created_at_ts = doc
            .get_first(self.created_at_field)
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as i64;

        let featured = doc
            .get_first(self.featured_field)
            .and_then(|v| v.as_u64())
            .unwrap_or(0) != 0;

        let active = doc
            .get_first(self.active_field)
            .and_then(|v| v.as_u64())
            .unwrap_or(0) != 0;

        let tags_str = doc
            .get_first(self.tags_field)
            .and_then(|v| v.as_text())
            .unwrap_or("");

        let tags: Vec<String> = if tags_str.is_empty() {
            Vec::new()
        } else {
            tags_str.split_whitespace().map(|s| s.to_string()).collect()
        };

        // Create DateTime from timestamp
        let created_at = DateTime::from_timestamp(created_at_ts, 0)
            .unwrap_or_else(|| Utc::now());

        // Note: This is a partial model since we only store searchable fields in the index
        // For complete model data, the caller should fetch from the primary storage
        let model = MarketplaceModel {
            model_id,
            owner,
            name,
            description,
            category,
            base_price,
            discount_price: base_price, // Not stored in search index
            minimum_bulk_size: 1,      // Not stored in search index
            framework,
            version: "unknown".to_string(), // Not stored in search index
            license,
            tags,
            input_shape: Vec::new(),  // Not stored in search index
            output_shape: Vec::new(), // Not stored in search index
            parameters: 0,            // Not stored in search index
            size_bytes: 0,           // Not stored in search index
            model_cid: String::new(), // Not stored in search index
            metadata_uri: String::new(), // Not stored in search index
            total_sales,
            total_revenue: 0,         // Not stored in search index
            rating,
            review_count: 0,          // Not stored in search index
            featured,
            active,
            created_at,
            updated_at: created_at,
            last_sale_at: None,
        };

        Ok(Some(model))
    }

    fn sort_results(&self, results: &mut [SearchResult], sort_by: SortBy) {
        match sort_by {
            SortBy::Relevance => {
                // Already sorted by relevance score
            }
            SortBy::Rating => {
                results.sort_by(|a, b| b.model.rating.partial_cmp(&a.model.rating).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortBy::Price => {
                results.sort_by(|a, b| a.model.base_price.cmp(&b.model.base_price));
            }
            SortBy::Sales => {
                results.sort_by(|a, b| b.model.total_sales.cmp(&a.model.total_sales));
            }
            SortBy::Newest => {
                results.sort_by(|a, b| b.model.created_at.cmp(&a.model.created_at));
            }
            SortBy::MostReviewed => {
                results.sort_by(|a, b| b.model.review_count.cmp(&a.model.review_count));
            }
            SortBy::Popularity => {
                results.sort_by(|a, b| {
                    let score_a = (a.model.total_sales as f32 * 0.6) + (a.model.rating * 0.4);
                    let score_b = (b.model.total_sales as f32 * 0.6) + (b.model.rating * 0.4);
                    score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
    }
}