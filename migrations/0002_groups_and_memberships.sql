CREATE TABLE IF NOT EXISTS groups (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(120) NOT NULL UNIQUE,
    description VARCHAR(500),
    created_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS memberships (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id BIGINT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    role VARCHAR(16) NOT NULL DEFAULT 'member' CHECK (role IN ('member', 'admin')),
    joined_at BIGINT NOT NULL,
    CONSTRAINT uniq_membership UNIQUE (user_id, group_id)
);

CREATE INDEX IF NOT EXISTS idx_memberships_group_id ON memberships(group_id);

