-- Add migration script here

CREATE TABLE IF NOT EXISTS posts 
(
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    content TEXT NOT NULL,
    author_id BIGINT REFERENCES users (id) ON DELETE CASCADE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

CREATE INDEX IF NOT EXISTS created_at_idx on posts (created_at);
CREATE INDEX IF NOT EXISTS author_id_idx on posts (author_id);