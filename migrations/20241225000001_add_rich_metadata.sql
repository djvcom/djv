-- Add commit_count to repositories
ALTER TABLE repositories ADD COLUMN commit_count INTEGER;

-- Drop and recreate the projects view with richer metadata
DROP VIEW IF EXISTS projects;

CREATE VIEW projects AS
-- Crates (preferred over their repos)
SELECT
    c.id,
    'crate'::TEXT AS kind,
    c.name,
    c.description,
    c.crates_io_url AS url,
    'rust'::TEXT AS language,
    c.keywords AS topics,
    c.downloads AS popularity,
    c.version,
    r.commit_count,
    r.updated_at,
    c.synced_at
FROM crates c
LEFT JOIN repositories r ON c.repository_id = r.id

UNION ALL

-- NPM packages (preferred over their repos)
SELECT
    n.id,
    'npm'::TEXT AS kind,
    n.name,
    n.description,
    n.npm_url AS url,
    'typescript'::TEXT AS language,
    n.keywords AS topics,
    n.downloads_weekly AS popularity,
    n.version,
    r.commit_count,
    r.updated_at,
    n.synced_at
FROM npm_packages n
LEFT JOIN repositories r ON n.repository_id = r.id

UNION ALL

-- Repositories not represented by crates/packages
SELECT
    r.id,
    'repo'::TEXT AS kind,
    r.name,
    r.description,
    r.url,
    r.language,
    r.topics,
    r.stars AS popularity,
    NULL::TEXT AS version,
    r.commit_count,
    r.updated_at,
    r.synced_at
FROM repositories r
WHERE NOT EXISTS (SELECT 1 FROM crates WHERE repository_id = r.id)
  AND NOT EXISTS (SELECT 1 FROM npm_packages WHERE repository_id = r.id);
