-- Create repositories table for git repos from forges (GitHub, Codeberg, etc.)
CREATE TABLE repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    forge TEXT NOT NULL,
    forge_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    url TEXT NOT NULL,
    language TEXT,
    stars INTEGER DEFAULT 0,
    topics TEXT[],
    updated_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (forge, forge_id)
);

CREATE INDEX idx_repositories_forge ON repositories(forge);
CREATE INDEX idx_repositories_language ON repositories(language) WHERE language IS NOT NULL;
