-- Create contributions table for open source contributions to external repos
CREATE TABLE contributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    forge TEXT NOT NULL,
    repo_owner TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    repo_url TEXT NOT NULL,
    contribution_type TEXT NOT NULL,
    title TEXT,
    url TEXT NOT NULL,
    merged_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (forge, repo_owner, repo_name, url)
);

CREATE INDEX idx_contributions_merged_at ON contributions(merged_at DESC) WHERE merged_at IS NOT NULL;
