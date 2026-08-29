CREATE TABLE IF NOT EXISTS scalar_fixtures (
    id TEXT PRIMARY KEY,
    external_id TEXT NOT NULL UNIQUE,
    small_value INTEGER NOT NULL,
    int_value INTEGER NOT NULL,
    big_value INTEGER NOT NULL,
    float_value REAL NOT NULL,
    double_value REAL NOT NULL,
    payload BLOB,
    metadata TEXT NOT NULL,
    amount REAL NOT NULL,
    local_time TEXT NOT NULL,
    event_time TEXT
);

CREATE TABLE IF NOT EXISTS locale_settings (
    tenant_id TEXT NOT NULL,
    locale TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (tenant_id, locale)
);
