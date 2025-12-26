-- Index for repository URL lookup (used in crates sync)
CREATE UNIQUE INDEX IF NOT EXISTS idx_repositories_url ON repositories(url);

-- GIN indexes for array queries (topic/keyword filtering)
CREATE INDEX IF NOT EXISTS idx_repositories_topics ON repositories USING GIN(topics)
  WHERE topics IS NOT NULL AND array_length(topics, 1) > 0;

CREATE INDEX IF NOT EXISTS idx_crates_keywords ON crates USING GIN(keywords)
  WHERE keywords IS NOT NULL AND array_length(keywords, 1) > 0;
