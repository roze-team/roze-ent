CREATE TABLE IF NOT EXISTS `roze_ent`.`audit_events` (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    action VARCHAR(120) NOT NULL,
    created_at BIGINT NOT NULL,
    CONSTRAINT fk_audit_event_user FOREIGN KEY (user_id) REFERENCES `roze_ent`.`users`(id) ON DELETE CASCADE
);

CREATE INDEX idx_audit_events_user_id ON `roze_ent`.`audit_events`(user_id);
