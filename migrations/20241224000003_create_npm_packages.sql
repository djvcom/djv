-- Create npm_packages table for NPM packages from npmjs.com
CREATE TABLE npm_packages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    scope TEXT,
    description TEXT,
    repository_id UUID REFERENCES repositories(id) ON DELETE SET NULL,
    npm_url TEXT NOT NULL,
    downloads_weekly INTEGER DEFAULT 0,
    version TEXT,
    keywords TEXT[],
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_npm_packages_repository_id ON npm_packages(repository_id) WHERE repository_id IS NOT NULL;
CREATE INDEX idx_npm_packages_downloads ON npm_packages(downloads_weekly DESC);
