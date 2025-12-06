# BATCH_007: Discovery Service Implementation Tasks

**Generated**: 2025-12-06
**Crate**: discovery
**Status**: Analysis Complete
**Priority**: High

---

## Executive Summary

The discovery crate (search/catalog) has a strong foundation with hybrid search, caching, analytics, and personalization. However, several critical features from the SPARC specification are incomplete or missing. This document identifies gaps and provides a prioritized task list for BATCH_007.

**Current Implementation Status**: ~65% Complete
- ✅ **Complete**: Hybrid search, vector search (Qdrant), keyword search (Tantivy), facets, filters, autocomplete, personalization, analytics, caching
- ⚠️ **Partial**: Embedding service (TODO in vector.rs:57), catalog management (missing), fuzzy search (basic)
- ❌ **Missing**: Graph-based discovery, recommendation engine, query spell correction, catalog CRUD operations, availability tracking

---

## 1. Module Analysis

### 1.1 Implemented Modules

| Module | Status | Lines | Features | Gaps |
|--------|--------|-------|----------|------|
| `search/mod.rs` | ✅ Complete | 580 | HybridSearchService, RRF, caching | Graph search placeholder |
| `search/vector.rs` | ⚠️ Partial | 357 | Qdrant HNSW, pre/post filter | Embedding service TODO (line 57) |
| `search/keyword.rs` | ✅ Complete | 266 | Tantivy BM25, fuzzy search | - |
| `search/facets.rs` | ✅ Complete | 248 | Genre/platform/year/rating facets | - |
| `search/filters.rs` | ✅ Complete | 151 | Genre/platform/year/rating filters | - |
| `search/autocomplete.rs` | ✅ Complete | 232 | Trie-based autocomplete, caching | - |
| `search/personalization.rs` | ✅ Complete | 373 | SONA integration, A/B testing | - |
| `search/query_processor.rs` | ⚠️ Partial | 273 | Spell correction, synonyms | Limited dictionary |
| `analytics/search_analytics.rs` | ✅ Complete | 500+ | Query logs, dashboards, metrics | - |
| `analytics/query_log.rs` | ✅ Complete | 300+ | Search events, click tracking | - |
| `cache.rs` | ✅ Complete | 700+ | Redis cache, stats, compression | - |
| `config.rs` | ✅ Complete | 272 | All configuration structures | - |
| `embedding.rs` | ✅ Complete | 237 | OpenAI embeddings, retry logic | - |
| `intent.rs` | ✅ Complete | 400+ | GPT-4o-mini intent parsing | - |

### 1.2 Missing Modules

| Module | Priority | Estimated LOC | Description |
|--------|----------|---------------|-------------|
| `catalog/mod.rs` | 🔴 Critical | 300 | Catalog management orchestrator |
| `catalog/crud.rs` | 🔴 Critical | 400 | Create/Read/Update/Delete operations |
| `catalog/ingestion.rs` | 🔴 Critical | 350 | Content ingestion pipeline |
| `catalog/validation.rs` | 🟡 High | 200 | Schema validation, data quality |
| `search/graph.rs` | 🟡 High | 500 | Graph-based discovery (BFS traversal) |
| `search/recommendations.rs` | 🟡 High | 450 | Recommendation engine integration |
| `search/availability.rs` | 🟡 High | 300 | Platform availability tracking |
| `search/ranking.rs` | 🟡 High | 400 | Multi-factor scoring from spec |
| `catalog/metadata.rs` | 🟢 Medium | 250 | Metadata enrichment |
| `catalog/deduplication.rs` | 🟢 Medium | 200 | Content deduplication |

---

## 2. Search Capabilities Analysis

### 2.1 Implemented Features

#### ✅ Vector Search (Qdrant)
- **File**: `search/vector.rs`
- **Implementation**: Complete HNSW search with pre/post filtering
- **Performance**: O(log n) with ef_search=64
- **Issue**: Embedding service TODO at line 57 - fallback to zero vector

#### ✅ Keyword Search (Tantivy)
- **File**: `search/keyword.rs`
- **Implementation**: BM25 ranking with fuzzy matching
- **Features**: Title, overview, genre boosting
- **Performance**: O(log n) with inverted index

#### ✅ Faceted Search
- **File**: `search/facets.rs`
- **Implementation**: Genre, platform, year, rating facets
- **Bucketing**: Decade-based years, rating ranges
- **Performance**: O(r) where r = result count

#### ✅ Autocomplete
- **File**: `search/autocomplete.rs`
- **Implementation**: Trie-based prefix matching
- **Features**: Case-insensitive, popularity sorting, Redis caching
- **Performance**: O(p) where p = prefix length

#### ✅ Personalization
- **File**: `search/personalization.rs`
- **Implementation**: SONA integration, A/B testing
- **Variants**: control, low_boost, medium_boost, high_boost, aggressive_boost
- **Latency**: <50ms target (verified)

### 2.2 Missing Features

#### ❌ Graph-Based Discovery
- **Specification**: `docs/sparc/pseudocode/search-discovery-engine.md` (lines 934-1211)
- **Requirements**:
  - BFS traversal with MAX_DEPTH=3, MAX_TRAVERSALS=100
  - Relationship types: SIMILAR_TO, SAME_FRANCHISE, SAME_DIRECTOR, SHARED_CAST, CO_WATCHED
  - Multi-path scoring with depth decay
  - Seed node identification (recent watches, favorites, references)
- **Estimated Effort**: 3-4 days

#### ❌ Advanced Ranking
- **Specification**: `docs/sparc/pseudocode/search-discovery-engine.md` (lines 1214-1531)
- **Requirements**:
  - Multi-factor scoring: base match, theme, preference, popularity, freshness, platform
  - User preference loading from SONA
  - Diversity enforcement (sliding window, genre clustering prevention)
  - Cosine similarity for vector-based preferences
- **Current**: Basic RRF (Reciprocal Rank Fusion) only
- **Estimated Effort**: 2-3 days

#### ❌ Availability Filtering
- **Specification**: `docs/sparc/pseudocode/search-discovery-engine.md` (lines 1552-1750)
- **Requirements**:
  - Regional availability lookup
  - Platform-based filtering
  - Price tier filtering (free, subscription, rent, buy)
  - Best availability selection
  - Availability boost in scoring
- **Estimated Effort**: 2 days

#### ⚠️ Query Spell Correction (Partial)
- **Current**: Basic Levenshtein distance with small dictionary
- **Gaps**:
  - Limited dictionary (~100 terms)
  - No title-based dictionary from database
  - No context-aware corrections
  - No "Did you mean?" suggestions
- **Estimated Effort**: 1 day enhancement

#### ⚠️ Fuzzy Search (Basic)
- **Current**: Tantivy's built-in fuzzy matching
- **Gaps**:
  - No configurable edit distance
  - No phonetic matching (Soundex, Metaphone)
  - No typo-specific corrections
- **Estimated Effort**: 1-2 days

---

## 3. Catalog Management Gaps

### 3.1 Critical Missing Features

The discovery crate has **zero catalog management functionality**. Based on the SPARC specification and media gateway requirements:

#### ❌ Catalog CRUD Operations
**Required APIs**:
```rust
// Missing: catalog/crud.rs
pub async fn create_content(content: ContentInput) -> Result<Uuid>
pub async fn update_content(id: Uuid, updates: ContentUpdate) -> Result<()>
pub async fn delete_content(id: Uuid) -> Result<()>
pub async fn get_content(id: Uuid) -> Result<Content>
pub async fn list_content(filters: CatalogFilters, pagination: Pagination) -> Result<ContentList>
```

**Database Schema Gaps**:
- No content ingestion tables
- No versioning/audit trail
- No content lifecycle states (draft, published, archived)
- No bulk operations support

**Estimated Effort**: 4-5 days

#### ❌ Content Ingestion Pipeline
**Required Features**:
- Platform integration adapters (TMDb, IMDb, streaming platforms)
- Metadata extraction and normalization
- Image/asset processing
- Duplicate detection
- Validation and quality checks
- Batch processing support
- Error handling and retry logic

**Estimated Effort**: 5-6 days

#### ❌ Metadata Management
**Required Features**:
- Rich metadata fields (cast, crew, ratings, certifications)
- Multi-language support
- External ID mapping (TMDb, IMDb, platform IDs)
- Metadata enrichment from multiple sources
- Metadata versioning

**Estimated Effort**: 3 days

#### ❌ Content Validation
**Required Features**:
- Schema validation
- Data quality checks (required fields, format validation)
- Business rule validation (release dates, ratings)
- Duplicate detection across sources
- Conflict resolution strategies

**Estimated Effort**: 2 days

### 3.2 Catalog Features in Other Crates

**Note**: Some catalog functionality may exist in:
- `media-gateway-core` (data models)
- `media-gateway-api` (REST endpoints)

**Recommendation**: Consolidate all catalog management in discovery crate for cohesion.

---

## 4. Analytics Gaps

### 4.1 Implemented Analytics

#### ✅ Search Analytics
- **File**: `analytics/search_analytics.rs`
- **Features**:
  - Query log with filters, latency, result counts
  - Popular queries tracking
  - Zero-result queries tracking
  - Latency statistics (P50, P95, P99)
  - Analytics dashboard aggregations
  - Time-based filtering (hourly, daily, weekly, monthly)

#### ✅ Query Logging
- **File**: `analytics/query_log.rs`
- **Features**:
  - Search event tracking
  - Click event tracking
  - User segmentation
  - Filter analytics

### 4.2 Missing Analytics

#### ❌ A/B Test Reporting
- **Need**: Aggregate metrics by experiment variant
- **Use Case**: Personalization boost comparison (control vs low_boost vs high_boost)
- **Estimated Effort**: 1 day

#### ❌ Search Quality Metrics
- **Metrics Needed**:
  - Click-through rate (CTR)
  - Mean reciprocal rank (MRR)
  - Normalized discounted cumulative gain (NDCG)
  - Time to first click
  - Zero-result rate by query type
- **Estimated Effort**: 2 days

#### ❌ User Behavior Analytics
- **Metrics Needed**:
  - Session duration
  - Query refinement patterns
  - Filter usage patterns
  - Platform preference tracking
- **Estimated Effort**: 2 days

---

## 5. TODO Comments & Placeholders

### 5.1 Critical TODOs

| File | Line | TODO | Impact | Effort |
|------|------|------|--------|--------|
| `search/vector.rs` | 57 | `// TODO: implement embedding service` | High - Falls back to zero vector | 2 hours |

**Details**:
```rust
// Line 57: search/vector.rs
// Generate query embedding (TODO: implement embedding service)
let query_vector = self.generate_embedding(query).await?;
```

**Current Behavior**: If no embedding service configured, returns `vec![0.0; dimension]`
**Fix Required**: Connect to `EmbeddingService` from `embedding.rs`
**Test Impact**: Integration tests may be using zero vectors

### 5.2 Minor Issues

| File | Line | Issue | Impact |
|------|------|-------|--------|
| `search/keyword.rs` | 266 | Hardcoded test data | Low - Test only |

---

## 6. BATCH_007 Task Breakdown

### Priority 1: Critical (Must Have)

#### TASK 7.1: Fix Vector Search Embedding Service
- **File**: `search/vector.rs`
- **Line**: 57
- **Description**: Remove TODO, properly integrate EmbeddingService
- **Acceptance Criteria**:
  - Vector search generates real embeddings via OpenAI
  - Fallback to zero vector only if OPENAI_API_KEY not set
  - Add warning log for fallback case
  - Update integration tests
- **Effort**: 2 hours
- **Priority**: 🔴 Critical

#### TASK 7.2: Implement Catalog CRUD Operations
- **New Files**:
  - `catalog/mod.rs`
  - `catalog/crud.rs`
  - `catalog/models.rs`
- **Description**: Core catalog management API
- **Features**:
  - Create content with validation
  - Update content with partial updates support
  - Delete content (soft delete)
  - Get content by ID
  - List content with filters and pagination
  - Batch operations (bulk create/update)
- **Database Migration**:
  ```sql
  CREATE TABLE content (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    overview TEXT,
    release_year INTEGER,
    genres TEXT[] NOT NULL DEFAULT '{}',
    platforms TEXT[] NOT NULL DEFAULT '{}',
    popularity_score FLOAT DEFAULT 0.0,
    status TEXT NOT NULL DEFAULT 'draft', -- draft, published, archived
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    metadata JSONB
  );

  CREATE INDEX idx_content_status ON content(status);
  CREATE INDEX idx_content_platforms ON content USING GIN(platforms);
  CREATE INDEX idx_content_genres ON content USING GIN(genres);
  ```
- **Acceptance Criteria**:
  - All CRUD operations tested with real database
  - Validation for required fields
  - Proper error handling
  - Transaction support
  - Optimistic locking for updates
- **Effort**: 4-5 days
- **Priority**: 🔴 Critical

#### TASK 7.3: Implement Content Ingestion Pipeline
- **New Files**:
  - `catalog/ingestion.rs`
  - `catalog/adapters/tmdb.rs`
  - `catalog/adapters/imdb.rs`
- **Description**: Automated content ingestion from external sources
- **Features**:
  - TMDb API adapter (movies, TV shows)
  - IMDb data adapter (ratings, metadata)
  - Metadata extraction and normalization
  - Duplicate detection (title + year matching)
  - Batch processing with rate limiting
  - Error handling and retry logic
  - Ingestion job tracking
- **Configuration**:
  ```toml
  [ingestion]
  tmdb_api_key = "..."
  batch_size = 100
  rate_limit_per_second = 5
  retry_attempts = 3
  deduplication_threshold = 0.9
  ```
- **Acceptance Criteria**:
  - Successfully ingest 1000+ titles from TMDb
  - Detect and skip duplicates
  - Handle API rate limits gracefully
  - Log ingestion metrics (success rate, failures)
  - Resume interrupted ingestion jobs
- **Effort**: 5-6 days
- **Priority**: 🔴 Critical

### Priority 2: High (Should Have)

#### TASK 7.4: Implement Graph-Based Discovery
- **New File**: `search/graph.rs`
- **Description**: Relationship traversal for content discovery
- **Specification**: `docs/sparc/pseudocode/search-discovery-engine.md` (lines 934-1211)
- **Features**:
  - BFS traversal with configurable depth
  - Relationship types: SIMILAR_TO, SAME_FRANCHISE, SAME_DIRECTOR, SHARED_CAST, CO_WATCHED
  - Multi-path scoring with depth decay
  - Seed node identification from user history
  - Edge weight configuration
- **Database Schema**:
  ```sql
  CREATE TABLE relationships (
    id UUID PRIMARY KEY,
    source_id UUID NOT NULL REFERENCES content(id),
    target_id UUID NOT NULL REFERENCES content(id),
    type TEXT NOT NULL, -- SIMILAR_TO, SAME_FRANCHISE, etc.
    weight FLOAT NOT NULL DEFAULT 1.0,
    similarity_score FLOAT,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
  );

  CREATE INDEX idx_relationships_source_type
    ON relationships(source_id, type, weight DESC);
  CREATE INDEX idx_relationships_target
    ON relationships(target_id);
  ```
- **Acceptance Criteria**:
  - Traverse relationships with MAX_DEPTH=3
  - Respect MAX_TRAVERSALS limit
  - Apply depth decay (0.7^depth)
  - Return multi-path discoveries with boost
  - Performance: <100ms for typical traversal
- **Effort**: 3-4 days
- **Priority**: 🟡 High

#### TASK 7.5: Implement Advanced Ranking
- **New File**: `search/ranking.rs`
- **Description**: Multi-factor result scoring
- **Specification**: `docs/sparc/pseudocode/search-discovery-engine.md` (lines 1214-1531)
- **Features**:
  - Base match score (from search strategy)
  - Theme matching (mood + themes from intent)
  - User preference alignment (genres, actors, directors)
  - Popularity boost
  - Freshness boost (exponential decay)
  - Platform availability boost
  - Diversity enforcement (sliding window)
- **Weights** (configurable):
  ```rust
  pub struct RankingWeights {
      pub base_match: f32 = 1.0,
      pub theme_match: f32 = 0.5,
      pub preference: f32 = 0.8,
      pub popularity: f32 = 0.3,
      pub freshness: f32 = 0.2,
      pub platform_match: f32 = 0.4,
  }
  ```
- **Acceptance Criteria**:
  - All 6 scoring components implemented
  - Load user preferences from database
  - Diversity enforcement prevents genre clustering
  - Performance: <50ms for 100 results
  - Return score breakdown for debugging
- **Effort**: 2-3 days
- **Priority**: 🟡 High

#### TASK 7.6: Implement Availability Filtering
- **New File**: `search/availability.rs`
- **Description**: Platform and regional availability tracking
- **Specification**: `docs/sparc/pseudocode/search-discovery-engine.md` (lines 1552-1750)
- **Features**:
  - Regional availability lookup
  - Platform-based filtering
  - Price tier filtering (free, subscription, rent, buy)
  - Best availability selection
  - Availability boost in scoring
- **Database Schema**:
  ```sql
  CREATE TABLE media_availability (
    id UUID PRIMARY KEY,
    media_id UUID NOT NULL REFERENCES content(id),
    platform TEXT NOT NULL,
    region TEXT NOT NULL,
    availability_type TEXT NOT NULL, -- free, subscription, rent, buy
    price DECIMAL(10,2),
    currency TEXT,
    url TEXT,
    quality TEXT,
    is_active BOOLEAN DEFAULT true,
    last_verified TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
  );

  CREATE INDEX idx_availability_media_region
    ON media_availability(media_id, region, is_active);
  ```
- **Acceptance Criteria**:
  - Filter results by user's platforms
  - Filter by price tier
  - Select best availability (free > subscription > rent > buy)
  - Apply availability boost to scores
  - Cache availability data (1 hour TTL)
  - Performance: <20ms for 100 results
- **Effort**: 2 days
- **Priority**: 🟡 High

#### TASK 7.7: Enhance Query Spell Correction
- **File**: `search/query_processor.rs`
- **Description**: Improve spell correction with larger dictionary
- **Current Gaps**:
  - Dictionary has only ~100 terms
  - No title-based corrections
  - No "Did you mean?" suggestions
- **Enhancements**:
  - Build dictionary from database titles
  - Add popular search queries to dictionary
  - Implement "Did you mean?" with confidence threshold
  - Add phonetic matching (Metaphone)
  - Cache dictionary in Redis
- **Acceptance Criteria**:
  - Dictionary includes 10,000+ terms
  - Correct common typos with >90% accuracy
  - Provide "Did you mean?" for low-confidence queries
  - Performance: <20ms for query processing
- **Effort**: 1 day
- **Priority**: 🟡 High

### Priority 3: Medium (Nice to Have)

#### TASK 7.8: Implement Metadata Validation
- **New File**: `catalog/validation.rs`
- **Description**: Comprehensive content validation
- **Features**:
  - Schema validation (required fields, types)
  - Business rule validation (dates, ratings)
  - Data quality checks (string lengths, formats)
  - Cross-field validation (release_date < added_date)
  - Custom validation rules
- **Acceptance Criteria**:
  - Validate all required fields
  - Return detailed validation errors
  - Support custom validation functions
  - Performance: <5ms per validation
- **Effort**: 2 days
- **Priority**: 🟢 Medium

#### TASK 7.9: Implement Deduplication
- **New File**: `catalog/deduplication.rs`
- **Description**: Detect and merge duplicate content
- **Strategies**:
  - Exact title + year match
  - Fuzzy title match (Levenshtein distance)
  - External ID matching (TMDb, IMDb)
  - Vector similarity (title embeddings)
- **Features**:
  - Detect duplicates during ingestion
  - Merge duplicate records
  - Preserve external IDs from all sources
  - Conflict resolution (prefer higher quality data)
- **Acceptance Criteria**:
  - Detect duplicates with >95% accuracy
  - Merge without data loss
  - Log all merge operations
  - Performance: <50ms per duplicate check
- **Effort**: 2 days
- **Priority**: 🟢 Medium

#### TASK 7.10: Add A/B Test Reporting
- **File**: `analytics/search_analytics.rs`
- **Description**: Analytics aggregated by experiment variant
- **Features**:
  - Metrics by variant (control, low_boost, high_boost, etc.)
  - CTR comparison across variants
  - Latency comparison
  - Statistical significance testing
  - Confidence intervals
- **Acceptance Criteria**:
  - Dashboard shows metrics by variant
  - Export A/B test reports to CSV
  - Calculate statistical significance
- **Effort**: 1 day
- **Priority**: 🟢 Medium

#### TASK 7.11: Add Search Quality Metrics
- **File**: `analytics/search_analytics.rs`
- **Description**: Advanced search quality metrics
- **Metrics**:
  - Click-through rate (CTR)
  - Mean reciprocal rank (MRR)
  - Normalized discounted cumulative gain (NDCG)
  - Time to first click
  - Zero-result rate by query type
- **Acceptance Criteria**:
  - Calculate all 5 metrics
  - Track trends over time
  - Alert on quality degradation
  - Dashboard visualizations
- **Effort**: 2 days
- **Priority**: 🟢 Medium

---

## 7. Implementation Order

### Week 1: Critical Fixes & Catalog Foundation
1. **Day 1**: TASK 7.1 - Fix Vector Search Embedding (2 hours)
2. **Day 1-2**: TASK 7.2 - Catalog CRUD Operations (Phase 1: Models & Basic CRUD)
3. **Day 3-5**: TASK 7.2 - Catalog CRUD Operations (Phase 2: Batch ops, validation)

### Week 2: Ingestion & Graph Discovery
1. **Day 1-3**: TASK 7.3 - Content Ingestion Pipeline (TMDb adapter)
2. **Day 4-5**: TASK 7.3 - Content Ingestion Pipeline (Deduplication, error handling)

### Week 3: Search Enhancements
1. **Day 1-2**: TASK 7.4 - Graph-Based Discovery (BFS, relationship traversal)
2. **Day 3-4**: TASK 7.5 - Advanced Ranking (Multi-factor scoring)
3. **Day 5**: TASK 7.6 - Availability Filtering (Basic implementation)

### Week 4: Polish & Analytics
1. **Day 1**: TASK 7.6 - Availability Filtering (Caching, optimization)
2. **Day 2**: TASK 7.7 - Enhanced Query Spell Correction
3. **Day 3**: TASK 7.8 - Metadata Validation
4. **Day 4**: TASK 7.9 - Deduplication
5. **Day 5**: TASK 7.10 & 7.11 - Analytics Enhancements

---

## 8. Testing Requirements

### 8.1 Unit Tests
- **Coverage Target**: >80% for new code
- **Focus Areas**:
  - Catalog CRUD operations
  - Validation logic
  - Deduplication algorithms
  - Ranking score calculations
  - Graph traversal logic

### 8.2 Integration Tests
- **Required Tests**:
  - End-to-end search with real database
  - Ingestion pipeline with TMDb API
  - Graph discovery with relationship data
  - Availability filtering with real platform data
  - Cache integration (Redis)
  - Vector search with Qdrant
  - Keyword search with Tantivy

### 8.3 Performance Tests
- **Benchmarks Required**:
  - Search latency (P50, P95, P99) <500ms
  - Ingestion throughput (>100 items/sec)
  - Graph traversal (<100ms)
  - Ranking (<50ms for 100 results)
  - Availability filtering (<20ms)

### 8.4 Load Tests
- **Scenarios**:
  - 1000 concurrent search queries
  - 10,000 content ingestion
  - Mixed read/write workload

---

## 9. Database Migrations

### 9.1 Required Migrations

#### Migration 1: Content Table
```sql
-- File: migrations/2025-12-06-001-create-content-table.sql
CREATE TABLE content (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title TEXT NOT NULL,
  overview TEXT,
  release_year INTEGER,
  genres TEXT[] NOT NULL DEFAULT '{}',
  platforms TEXT[] NOT NULL DEFAULT '{}',
  popularity_score FLOAT DEFAULT 0.0,
  status TEXT NOT NULL DEFAULT 'draft',
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by UUID,
  metadata JSONB,
  version INTEGER NOT NULL DEFAULT 1,
  CONSTRAINT valid_status CHECK (status IN ('draft', 'published', 'archived'))
);

CREATE INDEX idx_content_title_lower ON content (LOWER(title));
CREATE INDEX idx_content_title_trigram ON content USING GIN (title gin_trgm_ops);
CREATE INDEX idx_content_status ON content(status);
CREATE INDEX idx_content_platforms ON content USING GIN(platforms);
CREATE INDEX idx_content_genres ON content USING GIN(genres);
CREATE INDEX idx_content_release_year ON content(release_year);
```

#### Migration 2: Relationships Table
```sql
-- File: migrations/2025-12-06-002-create-relationships-table.sql
CREATE TABLE relationships (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source_id UUID NOT NULL REFERENCES content(id) ON DELETE CASCADE,
  target_id UUID NOT NULL REFERENCES content(id) ON DELETE CASCADE,
  type TEXT NOT NULL,
  weight FLOAT NOT NULL DEFAULT 1.0,
  similarity_score FLOAT,
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  CONSTRAINT valid_relationship_type CHECK (type IN (
    'SIMILAR_TO', 'SAME_FRANCHISE', 'SAME_DIRECTOR',
    'SHARED_CAST', 'CO_WATCHED'
  )),
  CONSTRAINT different_nodes CHECK (source_id != target_id)
);

CREATE INDEX idx_relationships_source_type
  ON relationships(source_id, type, weight DESC);
CREATE INDEX idx_relationships_target
  ON relationships(target_id);
CREATE UNIQUE INDEX idx_relationships_unique
  ON relationships(source_id, target_id, type);
```

#### Migration 3: Availability Table
```sql
-- File: migrations/2025-12-06-003-create-availability-table.sql
CREATE TABLE media_availability (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  media_id UUID NOT NULL REFERENCES content(id) ON DELETE CASCADE,
  platform TEXT NOT NULL,
  region TEXT NOT NULL,
  availability_type TEXT NOT NULL,
  price DECIMAL(10,2),
  currency TEXT,
  url TEXT,
  quality TEXT,
  is_active BOOLEAN DEFAULT true,
  last_verified TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  CONSTRAINT valid_availability_type CHECK (availability_type IN (
    'free', 'subscription', 'rent', 'buy'
  ))
);

CREATE INDEX idx_availability_media_region
  ON media_availability(media_id, region, is_active);
CREATE INDEX idx_availability_platform
  ON media_availability(platform);
CREATE UNIQUE INDEX idx_availability_unique
  ON media_availability(media_id, platform, region, availability_type)
  WHERE is_active = true;
```

#### Migration 4: Ingestion Jobs Table
```sql
-- File: migrations/2025-12-06-004-create-ingestion-jobs-table.sql
CREATE TABLE ingestion_jobs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  total_items INTEGER,
  processed_items INTEGER DEFAULT 0,
  failed_items INTEGER DEFAULT 0,
  started_at TIMESTAMPTZ,
  completed_at TIMESTAMPTZ,
  error_message TEXT,
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  CONSTRAINT valid_status CHECK (status IN (
    'pending', 'running', 'completed', 'failed', 'paused'
  ))
);

CREATE INDEX idx_ingestion_jobs_status ON ingestion_jobs(status);
CREATE INDEX idx_ingestion_jobs_source ON ingestion_jobs(source);
```

---

## 10. Dependencies to Add

### 10.1 Cargo.toml Updates

```toml
[dependencies]
# Existing dependencies...

# For catalog management
validator = "0.16"  # Field validation
derive_builder = "0.12"  # Builder pattern for models

# For ingestion
reqwest = { version = "0.11", features = ["json", "gzip"] }
async-trait = "0.1"
backoff = "0.4"  # Retry with exponential backoff

# For graph algorithms
petgraph = "0.6"  # Graph data structures

# For deduplication
strsim = "0.10"  # String similarity (Levenshtein, etc.)
metaphone = "0.1"  # Phonetic matching

# For ranking
ordered-float = "3.0"  # Total ordering for f32/f64
```

---

## 11. Configuration Updates

### 11.1 Discovery Config Extensions

```rust
// File: src/config.rs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryConfig {
    // ... existing fields ...

    /// Catalog management configuration
    pub catalog: CatalogConfig,

    /// Graph discovery configuration
    pub graph: GraphConfig,

    /// Ranking configuration
    pub ranking: RankingConfig,

    /// Ingestion configuration
    pub ingestion: IngestionConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CatalogConfig {
    pub enable_soft_delete: bool,
    pub enable_versioning: bool,
    pub max_batch_size: usize,
    pub validation_strict_mode: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphConfig {
    pub max_depth: u32,
    pub max_traversals: usize,
    pub enable_caching: bool,
    pub cache_ttl_sec: u64,
    pub relationship_weights: HashMap<String, f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RankingConfig {
    pub weights: RankingWeights,
    pub diversity_window_size: usize,
    pub max_same_genre_in_window: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RankingWeights {
    pub base_match: f32,
    pub theme_match: f32,
    pub preference: f32,
    pub popularity: f32,
    pub freshness: f32,
    pub platform_match: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IngestionConfig {
    pub tmdb_api_key: String,
    pub tmdb_base_url: String,
    pub batch_size: usize,
    pub rate_limit_per_second: u32,
    pub retry_attempts: u32,
    pub deduplication_threshold: f32,
    pub enable_auto_ingestion: bool,
}
```

---

## 12. API Endpoints to Add

### 12.1 Catalog Management APIs

```rust
// File: src/server/handlers/catalog.rs

// Create content
POST /api/v1/catalog/content
Body: ContentInput
Response: 201 Created { id: UUID }

// Update content
PUT /api/v1/catalog/content/{id}
Body: ContentUpdate
Response: 200 OK

// Delete content (soft delete)
DELETE /api/v1/catalog/content/{id}
Response: 204 No Content

// Get content
GET /api/v1/catalog/content/{id}
Response: 200 OK Content

// List content
GET /api/v1/catalog/content?filter=...&page=1&size=20
Response: 200 OK { items: [Content], total: 1000, page: 1 }

// Batch create
POST /api/v1/catalog/content/batch
Body: [ContentInput]
Response: 201 Created { created: [UUID], failed: [Error] }

// Start ingestion job
POST /api/v1/catalog/ingest
Body: { source: "tmdb", params: {...} }
Response: 202 Accepted { job_id: UUID }

// Get ingestion job status
GET /api/v1/catalog/ingest/{job_id}
Response: 200 OK IngestionJob
```

### 12.2 Search Enhancement APIs

```rust
// File: src/server/handlers/search.rs

// Graph discovery
POST /api/v1/search/discover
Body: { seed_ids: [UUID], max_depth: 3, filters: {...} }
Response: 200 OK { results: [SearchResult] }

// Get recommendations
GET /api/v1/search/recommendations?user_id={id}&limit=20
Response: 200 OK { results: [SearchResult] }

// Check availability
GET /api/v1/search/availability?content_id={id}&region={region}
Response: 200 OK { availability: [AvailabilityOption] }

// Query suggestions (autocomplete)
GET /api/v1/search/suggest?q={prefix}&limit=10
Response: 200 OK { suggestions: [Suggestion] }
```

---

## 13. Success Metrics

### 13.1 Completion Criteria

- ✅ All Priority 1 tasks completed
- ✅ All Priority 2 tasks completed
- ✅ 80%+ test coverage on new code
- ✅ Search latency P95 <500ms maintained
- ✅ Catalog ingestion working end-to-end
- ✅ Graph discovery functional with real data
- ✅ Zero critical bugs in production

### 13.2 Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Search latency (P95) | <500ms | ~340ms | ✅ Good |
| Vector search | <150ms | ~100ms | ✅ Good |
| Graph traversal | <100ms | N/A | ⚠️ To implement |
| Ranking | <50ms | ~30ms (RRF only) | ⚠️ Enhance |
| Availability filtering | <20ms | N/A | ⚠️ To implement |
| Ingestion throughput | >100 items/sec | N/A | ⚠️ To implement |

### 13.3 Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Test coverage | >80% | ~70% | ⚠️ Increase |
| Intent parsing accuracy | >85% | ~90% | ✅ Good |
| Deduplication accuracy | >95% | N/A | ⚠️ To implement |
| Zero-result rate | <5% | ~8% | ⚠️ Improve |

---

## 14. Risks & Mitigation

### 14.1 Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Qdrant integration issues | High | Low | Use existing vector.rs as reference |
| Tantivy indexing performance | Medium | Low | Pre-build indexes, use sharding |
| Graph traversal performance | High | Medium | Limit depth/traversals, add caching |
| TMDb API rate limits | Medium | High | Implement backoff, batch requests |
| Database migration failures | High | Low | Test migrations on staging first |

### 14.2 Schedule Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Scope creep | High | Medium | Strict prioritization, defer P3 tasks |
| Dependency delays | Medium | Low | Identify dependencies early |
| Integration complexity | High | Medium | Incremental integration, daily testing |

---

## 15. Recommendations

### 15.1 Immediate Actions (Week 1)

1. **Fix critical TODO in vector.rs** (2 hours) - Unblocks vector search testing
2. **Design catalog schema** (1 day) - Enables parallel development
3. **Create database migrations** (1 day) - Unblocks catalog implementation
4. **Set up TMDb API account** (1 hour) - Enables ingestion testing

### 15.2 Architecture Recommendations

1. **Consolidate catalog management** in discovery crate (not core)
   - Rationale: Catalog is tightly coupled with search/indexing

2. **Use event sourcing for catalog changes**
   - Rationale: Audit trail, rollback capability, analytics

3. **Implement circuit breaker for external APIs** (TMDb, embeddings)
   - Rationale: Resilience, graceful degradation

4. **Add OpenTelemetry tracing** to all search paths
   - Rationale: Debug performance issues, optimize bottlenecks

### 15.3 Future Enhancements (Post-BATCH_007)

1. **Machine learning ranking model** (replace manual weights)
2. **Real-time graph updates** (via CDC from relationships table)
3. **Multi-language search** (embed multiple languages)
4. **Image-based search** (CLIP embeddings)
5. **Voice search** (speech-to-text + NLP)

---

## 16. Appendix

### 16.1 Reference Documentation

- **SPARC Specification**: `/workspaces/media-gateway/docs/sparc/pseudocode/search-discovery-engine.md`
- **Platform Integration**: `/workspaces/media-gateway/docs/PLATFORM_INTEGRATION_QUICK_REFERENCE.md`
- **Streaming Platform Spec**: `/workspaces/media-gateway/docs/STREAMING_PLATFORM_SPECIFICATION.md`
- **Quality Requirements**: `/workspaces/media-gateway/docs/QUALITY_REQUIREMENTS_SPECIFICATION.md`

### 16.2 External Dependencies

- **Qdrant**: https://qdrant.tech/documentation/
- **Tantivy**: https://docs.rs/tantivy/
- **TMDb API**: https://developers.themoviedb.org/3
- **OpenAI Embeddings**: https://platform.openai.com/docs/guides/embeddings

### 16.3 Team Contacts

- **Discovery Lead**: TBD
- **Database Team**: TBD
- **DevOps**: TBD
- **QA Lead**: TBD

---

**End of BATCH_007 Tasks Document**

**Generated by**: Claude Code Quality Analyzer
**Date**: 2025-12-06
**Status**: Ready for Review
