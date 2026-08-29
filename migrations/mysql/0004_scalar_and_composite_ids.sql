CREATE TABLE IF NOT EXISTS scalar_fixtures (
    id VARCHAR(64) PRIMARY KEY,
    external_id CHAR(36) NOT NULL UNIQUE,
    small_value SMALLINT NOT NULL,
    int_value INTEGER NOT NULL,
    big_value BIGINT NOT NULL,
    float_value FLOAT NOT NULL,
    double_value DOUBLE NOT NULL,
    payload BLOB,
    metadata JSON NOT NULL,
    amount DECIMAL(38, 12) NOT NULL,
    local_time DATETIME(6) NOT NULL,
    event_time TIMESTAMP(6) NULL
);

CREATE TABLE IF NOT EXISTS locale_settings (
    tenant_id VARCHAR(64) NOT NULL,
    locale VARCHAR(32) NOT NULL,
    value VARCHAR(500) NOT NULL,
    PRIMARY KEY (tenant_id, locale)
);
