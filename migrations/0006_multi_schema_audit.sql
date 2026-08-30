CREATE SCHEMA IF NOT EXISTS roze_ent;

CREATE TABLE IF NOT EXISTS roze_ent.audit_events (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    action VARCHAR(120) NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_events_user_id
    ON roze_ent.audit_events(user_id);
