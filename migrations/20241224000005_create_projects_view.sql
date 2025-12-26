-- Create unified projects view for UI display
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
    c.synced_at
FROM crates c

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
    n.synced_at
FROM npm_packages n

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
    r.synced_at
FROM repositories r
WHERE NOT EXISTS (SELECT 1 FROM crates WHERE repository_id = r.id)
  AND NOT EXISTS (SELECT 1 FROM npm_packages WHERE repository_id = r.id);
