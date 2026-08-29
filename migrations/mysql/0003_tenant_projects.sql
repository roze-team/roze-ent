CREATE TABLE IF NOT EXISTS projects (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    tenant_id VARCHAR(64) NOT NULL,
    name VARCHAR(120) NOT NULL,
    description VARCHAR(500),
    version BIGINT NOT NULL DEFAULT 1 CHECK (version > 0),
    deleted_at BIGINT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    CONSTRAINT uniq_project_tenant_name UNIQUE (tenant_id, name)
);

CREATE INDEX idx_projects_tenant_live ON projects(tenant_id, deleted_at, id);
