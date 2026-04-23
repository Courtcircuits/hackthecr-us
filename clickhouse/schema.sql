-- ============================================================
-- HackTheCrous – ClickHouse schema
-- ============================================================
-- Reputation model: every scraping job is sent to N scrapers.
-- For each pair of scrapers that answered the same job we
-- compute the Jaccard similarity of their result sets.
-- A scraper whose results consistently diverge from the
-- majority gets a lower reputation and its batches are denied.
--
-- Jaccard(A, B) = |A ∩ B| / |A ∪ B|
-- where A and B are the sets of entity keys returned by each
-- scraper for the same job.
-- ============================================================

-- ------------------------------------------------------------
-- 1. scraping_batches
--    One row per (job, scraper) submission.
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS scraping_batches
(
    batch_id     UUID,
    job_id       UUID,
    scraper_id   String,                    -- SignedPayload.author
    region       LowCardinality(String),
    entity       LowCardinality(String),    -- 'restaurants' | 'meals-<id>' | 'schools'
    submitted_at DateTime     DEFAULT now(),
    accepted     UInt8        DEFAULT 0     -- 1 once consensus is reached
) ENGINE = MergeTree()
ORDER BY (job_id, submitted_at)
PARTITION BY toYYYYMM(submitted_at);


-- ------------------------------------------------------------
-- 2a. scraped_restaurants
--     One row per restaurant per batch.
--     restaurant_id is the natural key used for Jaccard sets.
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS scraped_restaurants
(
    batch_id      UUID,
    job_id        UUID,
    scraper_id    String,
    region        LowCardinality(String),
    restaurant_id String,
    name          String,
    url           String,
    city          Nullable(String),
    coordinates   Nullable(String),
    opening_hours Nullable(String),
    scraped_at    DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (batch_id, restaurant_id)
PARTITION BY toYYYYMM(scraped_at);


-- ------------------------------------------------------------
-- 2b. scraped_meals
--     One row per meal per batch.
--     (restaurant_id, meal_type, date) is the natural key.
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS scraped_meals
(
    batch_id      UUID,
    job_id        UUID,
    scraper_id    String,
    region        LowCardinality(String),
    restaurant_id String,
    meal_type     LowCardinality(String),
    foodies       Nullable(String),
    date          Nullable(String),
    scraped_at    DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (batch_id, restaurant_id, meal_type)
PARTITION BY toYYYYMM(scraped_at);


-- ------------------------------------------------------------
-- 3. batch_jaccard_scores
--    Pairwise Jaccard similarity between every two scrapers
--    that answered the same job. Inserted by the buffet
--    service once it has collected enough batches for a job.
--    scraper_a < scraper_b (lexicographic) to avoid duplicates.
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS batch_jaccard_scores
(
    job_id      UUID,
    entity      LowCardinality(String),
    scraper_a   String,
    scraper_b   String,
    jaccard     Float32,   -- [0, 1]
    computed_at DateTime DEFAULT now()
) ENGINE = MergeTree()
ORDER BY (job_id, scraper_a, scraper_b)
PARTITION BY toYYYYMM(computed_at);


-- ------------------------------------------------------------
-- 4. scraper_reputation  (AggregatingMergeTree)
--    Incrementally maintained by a materialized view.
--    Each scraper appears once; merges are handled by the
--    engine so reads always see up-to-date aggregates.
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS scraper_reputation
(
    scraper_id   String,
    comparisons  AggregateFunction(count),
    jaccard_sum  AggregateFunction(sum, Float64),
    last_active  SimpleAggregateFunction(max, DateTime)
) ENGINE = AggregatingMergeTree()
ORDER BY scraper_id;

-- Materialized view: feed reputation from scraper_a side.
-- A symmetric view for scraper_b is added below so every
-- participant in a comparison gets credited.
CREATE MATERIALIZED VIEW IF NOT EXISTS scraper_reputation_mv_a
TO scraper_reputation AS
SELECT
    scraper_a                       AS scraper_id,
    countState()                    AS comparisons,
    sumState(toFloat64(jaccard))    AS jaccard_sum,
    maxSimpleState(computed_at)     AS last_active
FROM batch_jaccard_scores
GROUP BY scraper_a;

CREATE MATERIALIZED VIEW IF NOT EXISTS scraper_reputation_mv_b
TO scraper_reputation AS
SELECT
    scraper_b                       AS scraper_id,
    countState()                    AS comparisons,
    sumState(toFloat64(jaccard))    AS jaccard_sum,
    maxSimpleState(computed_at)     AS last_active
FROM batch_jaccard_scores
GROUP BY scraper_b;


-- ------------------------------------------------------------
-- 5. scraper_reputation_view  (query helper)
--    Use this for dashboards and admission decisions.
--    avg_jaccard close to 1 = trustworthy scraper.
--    avg_jaccard below a threshold (e.g. 0.6) = rogue.
-- ------------------------------------------------------------
CREATE VIEW IF NOT EXISTS scraper_reputation_view AS
SELECT
    scraper_id,
    countMerge(comparisons)                                         AS total_comparisons,
    sumMerge(jaccard_sum) / nullIf(countMerge(comparisons), 0)     AS avg_jaccard,
    max(last_active)                                                AS last_active,
    -- convenience flag: below 0.6 avg Jaccard is considered rogue
    if(avg_jaccard < 0.6, 1, 0)                                    AS is_rogue
FROM scraper_reputation
GROUP BY scraper_id
ORDER BY avg_jaccard DESC;


-- ------------------------------------------------------------
-- 6. Jaccard computation query (run by the buffet service
--    after collecting all batches for a job)
--
-- For restaurants: the set is the array of restaurant_ids.
-- For meals:       the set is the array of (restaurant_id, meal_type, date).
--
-- Example for restaurants:
-- ------------------------------------------------------------
--
-- INSERT INTO batch_jaccard_scores
-- SELECT
--     a.job_id,
--     'restaurants'                                                      AS entity,
--     if(a.scraper_id < b.scraper_id, a.scraper_id, b.scraper_id)       AS scraper_a,
--     if(a.scraper_id < b.scraper_id, b.scraper_id, a.scraper_id)       AS scraper_b,
--     length(arrayIntersect(a.ids, b.ids))
--         / length(arrayDistinct(arrayConcat(a.ids, b.ids)))             AS jaccard,
--     now()                                                              AS computed_at
-- FROM (
--     SELECT job_id, scraper_id, groupArray(restaurant_id) AS ids
--     FROM scraped_restaurants
--     WHERE job_id = '...'
--     GROUP BY job_id, scraper_id
-- ) a
-- JOIN (
--     SELECT job_id, scraper_id, groupArray(restaurant_id) AS ids
--     FROM scraped_restaurants
--     WHERE job_id = '...'
--     GROUP BY job_id, scraper_id
-- ) b USING (job_id)
-- WHERE a.scraper_id < b.scraper_id;
