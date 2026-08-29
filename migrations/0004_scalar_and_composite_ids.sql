CREATE TABLE IF NOT EXISTS scalar_fixtures (
    id VARCHAR(64) PRIMARY KEY,
    external_id UUID NOT NULL UNIQUE,
    small_value SMALLINT NOT NULL,
    int_value INTEGER NOT NULL,
    big_value BIGINT NOT NULL,
    float_value REAL NOT NULL,
    double_value DOUBLE PRECISION NOT NULL,
    payload BYTEA,
    metadata JSONB NOT NULL,
    amount NUMERIC NOT NULL,
    local_time TIMESTAMP NOT NULL,
    event_time TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS locale_settings (
    tenant_id VARCHAR(64) NOT NULL,
    locale VARCHAR(32) NOT NULL,
    value VARCHAR(500) NOT NULL,
    PRIMARY KEY (tenant_id, locale)
);
