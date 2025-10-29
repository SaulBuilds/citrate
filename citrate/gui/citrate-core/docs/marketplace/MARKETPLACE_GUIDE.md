# Citrate Model Marketplace - User Guide

## Overview

The Citrate Model Marketplace is a comprehensive platform for discovering, evaluating, and purchasing AI models. This guide covers all features available to users and model creators.

## Table of Contents

1. [Searching for Models](#searching-for-models)
2. [Using Filters and Sorting](#using-filters-and-sorting)
3. [Understanding Quality Scores](#understanding-quality-scores)
4. [Reading and Writing Reviews](#reading-and-writing-reviews)
5. [Viewing Performance Metrics](#viewing-performance-metrics)
6. [Using Recommendations](#using-recommendations)
7. [Editing Model Metadata (Creators)](#editing-model-metadata)
8. [IPFS Metadata Storage](#ipfs-metadata-storage)

---

## Searching for Models

### Basic Text Search

The marketplace uses FlexSearch for fast, typo-tolerant full-text search:

```
# Search examples:
"GPT language model"      → Finds GPT-based language models
"vision transformer"      → Finds vision models using transformers
"code completion"         → Finds code generation models
```

**Search Features:**
- **Typo tolerance**: "langage model" still finds "language model"
- **Multi-field search**: Searches name, description, tags, creator
- **Weighted relevance**: Model names ranked higher than descriptions
- **Real-time results**: Sub-500ms response time

### Advanced Search

Use filters for precise results:
- Category filters (Language, Vision, Code, etc.)
- Price range (min/max in ETH)
- Rating threshold
- Framework (PyTorch, TensorFlow, JAX)
- Model size (Tiny, Small, Medium, Large, X-Large)

---

## Using Filters and Sorting

### Available Filters

**Category Filter**
- Language Models
- Code Models
- Vision Models
- Embedding Models
- Multimodal Models
- Generative Models
- Audio Models
- Reinforcement Learning
- Time Series
- Tabular Models

**Price Range**
- Set minimum and maximum price in ETH
- Filter by discount availability

**Rating Filter**
- Minimum star rating (1-5 stars)
- Filter by number of reviews

**Framework Filter**
- PyTorch
- TensorFlow
- JAX
- ONNX
- Other

**Model Size Filter**
- Tiny (< 1GB)
- Small (1-5GB)
- Medium (5-20GB)
- Large (20-100GB)
- X-Large (> 100GB)

### Sorting Options

1. **Relevance** (default for text search) - Best match first
2. **Rating (High to Low)** - Highest rated models first
3. **Rating (Low to High)** - Lowest rated models first
4. **Price (High to Low)** - Most expensive first
5. **Price (Low to High)** - Cheapest first
6. **Popularity** - Most sales + inferences
7. **Recent** - Recently listed models
8. **Trending** - Hot models with recent activity

---

## Understanding Quality Scores

### Quality Score Breakdown (0-100)

The overall quality score is calculated from four components:

#### 1. Rating Score (40% weight)
- Based on user reviews (0-5 stars)
- Considers total number of reviews
- Weighs recent reviews higher
- Trend analysis (improving/declining)

**Calculation:**
```
Rating Score = (Average Stars / 5) * 100 * (log(reviews + 1) / log(100))
```

#### 2. Performance Score (30% weight)
- Average inference latency
- Reliability (uptime percentage)
- Consistency (low variance)

**Metrics:**
- Avg Latency: Lower is better
- Reliability: 99%+ uptime = excellent
- Consistency: Low std deviation = reliable

#### 3. Reliability Score (20% weight)
- Uptime percentage
- Error rate
- Mean time between failures
- Incident count

**Thresholds:**
- 99.9% uptime = 100 points
- < 1% error rate = excellent
- > 720 hours MTBF = excellent

#### 4. Engagement Score (10% weight)
- Total sales
- Total inferences
- Active users
- Growth rate

**Engagement Indicators:**
- High sales → Popular model
- High inferences → Actively used
- Growing → Trending up

### Interpreting Scores

- **90-100**: Exceptional quality
- **80-89**: Excellent
- **70-79**: Good
- **60-69**: Fair
- **Below 60**: Needs improvement

---

## Reading and Writing Reviews

### Writing a Review

**Requirements:**
- Must have purchased the model
- One review per user per model
- Can edit your own review anytime

**Review Components:**
1. **Star Rating** (1-5 stars, required)
2. **Title** (max 100 characters, required)
3. **Content** (max 2000 characters, required)
4. **Pros/Cons** (optional)
5. **Use Case** (optional)

**Best Practices:**
- Be specific about your use case
- Mention performance characteristics
- Include both pros and cons
- Provide constructive feedback

### Reading Reviews

**Review Display:**
- Sorted by helpfulness (default)
- Verified purchase badge
- Review date
- Helpful votes
- Creator responses

**Filters:**
- By star rating
- Verified purchases only
- Recent reviews
- Most helpful

### Review Moderation

Reviews are subject to moderation for:
- Spam
- Inappropriate content
- Fake reviews
- Off-topic content

Moderators can flag, edit, or remove reviews.

---

## Viewing Performance Metrics

### Metrics Dashboard

Access detailed performance data for each model:

#### Latency Metrics
- Average latency
- P50, P90, P95, P99 percentiles
- Min/Max latency
- Latency over time (chart)

#### Throughput Metrics
- Total inferences
- Successful inferences
- Failed inferences
- Throughput per second/minute/hour

#### Reliability Metrics
- Uptime percentage
- Error rate
- Mean time between failures
- Recent incidents

#### Engagement Metrics
- Total sales
- Total revenue
- Active users
- Returning users
- Growth rate

### Time Range Selection

View metrics for:
- Last 24 hours
- Last 7 days
- Last 30 days
- Last 90 days
- All time

### Exporting Metrics

Export formats:
- JSON (raw data)
- CSV (spreadsheet)
- XLSX (Excel)

Options:
- Include raw data points
- Include aggregates
- Include charts

---

## Using Recommendations

### Recommendation Types

#### 1. Similar Models
- Content-based filtering
- Based on category, tags, framework
- Excludes already purchased models

#### 2. Trending Models
- Recent activity-based ranking
- Sales + inference velocity
- Time-decay weighting

#### 3. Recently Viewed
- Your browsing history
- Stored locally (privacy-conscious)
- Clear history anytime

#### 4. Collaborative Recommendations
- "Users who bought this also bought..."
- Based on co-purchase patterns
- Shows co-occurrence percentage

#### 5. Personalized Recommendations
- Based on your purchase history
- Category and tag preferences
- Price range compatibility
- Diverse results

### Managing Recommendations

**Privacy Controls:**
- All tracking is local-only
- Clear history anytime
- Export your data (GDPR)
- Import/export history

**Recommendation Settings:**
- Enable/disable tracking
- Adjust diversity factor
- Set preferred categories

---

## Editing Model Metadata (Creators)

### Metadata Fields

**Basic Information:**
- Model name
- Description (markdown supported)
- Category
- Tags (comma-separated)

**Technical Details:**
- Framework
- Model size (bytes)
- Version
- Requirements

**Pricing:**
- Base price (ETH)
- Discount price (optional)

**Rich Metadata:**
- Architecture details
- Training data info
- Performance benchmarks
- Example usage
- Documentation links

### Updating Metadata

1. Navigate to your model page
2. Click "Edit Metadata"
3. Make changes in the editor
4. Preview changes
5. Upload to IPFS
6. Confirm transaction

**Note:** Metadata updates require a blockchain transaction to update the IPFS URI.

### Metadata Best Practices

- **Clear description**: Explain what your model does
- **Accurate tags**: Help users find your model
- **Detailed specs**: List requirements and limitations
- **Example usage**: Provide code examples
- **Documentation**: Link to comprehensive docs
- **Benchmarks**: Include performance numbers

---

## IPFS Metadata Storage

### How It Works

Model metadata is stored on IPFS for:
- **Decentralization**: No single point of failure
- **Permanence**: Content-addressed storage
- **Integrity**: CID verifies content
- **Availability**: Pinning services ensure access

### IPFS Flow

1. **Create Metadata**: Fill in model details
2. **Validate**: Check for required fields
3. **Upload to IPFS**: Returns CID
4. **Pin Content**: Ensure availability
5. **Store CID**: Save on blockchain

### Accessing Metadata

**IPFS Gateways:**
- Public: `https://ipfs.io/ipfs/{CID}`
- Cloudflare: `https://cloudflare-ipfs.com/ipfs/{CID}`
- Local: `http://localhost:8080/ipfs/{CID}`

**Direct Access:**
```bash
ipfs cat {CID}
```

### Pinning Services

Recommended pinning services:
- **Pinata**: pinata.cloud
- **NFT.Storage**: nft.storage
- **Fleek**: fleek.co
- **Infura**: infura.io

### Metadata Schema

JSON schema for model metadata:

```json
{
  "name": "Model Name",
  "description": "Detailed description",
  "category": "language-models",
  "tags": ["gpt", "language", "nlp"],
  "framework": "PyTorch",
  "version": "1.0.0",
  "modelSize": "5GB",
  "architecture": "Transformer",
  "trainingData": "Details about training data",
  "license": "MIT",
  "documentation": "https://docs.example.com",
  "examples": [
    {
      "title": "Basic Usage",
      "code": "import model..."
    }
  ],
  "benchmarks": [
    {
      "metric": "Accuracy",
      "value": "95.2%",
      "dataset": "ImageNet"
    }
  ]
}
```

---

## FAQ

**Q: How do I know if a model is good quality?**
A: Check the quality score breakdown, read reviews, and view performance metrics.

**Q: Can I try a model before buying?**
A: Some models offer free inference trials. Check the model page for details.

**Q: How do I report a problematic model or review?**
A: Use the "Report" button on the model or review page.

**Q: Can I get a refund?**
A: Refund policies vary by creator. Contact support for assistance.

**Q: How are recommendations personalized?**
A: Based on your viewing and purchase history, stored locally in your browser.

**Q: Can creators see my viewing history?**
A: No, all tracking is local-only for privacy.

---

## Support

For help and support:
- Discord: [discord.gg/citrate](https://discord.gg/citrate)
- Documentation: [docs.citrate.ai](https://docs.citrate.ai)
- Email: support@citrate.ai

---

## Changelog

### Version 1.0.0 (Sprint 4, Week 5)
- Initial marketplace release
- Search with filters and sorting
- Quality scores and metrics
- Review system
- Recommendation engine
- IPFS metadata storage
- Model card editor
