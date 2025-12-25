-- Create crates table for Rust crates from crates.io
CREATE TABLE crates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    repository_id UUID REFERENCES repositories(id) ON DELETE SET NULL,
    crates_io_url TEXT NOT NULL,
    documentation_url TEXT,
    downloads INTEGER DEFAULT 0,
    version TEXT,
    keywords TEXT[],
    categories TEXT[],
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_crates_repository_id ON crates(repository_id) WHERE repository_id IS NOT NULL;
CREATE INDEX idx_crates_downloads ON crates(downloads DESC);
