CREATE TABLE IF NOT EXISTS `groups` (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(120) NOT NULL UNIQUE,
    description VARCHAR(500),
    created_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS memberships (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    group_id BIGINT NOT NULL,
    role VARCHAR(16) NOT NULL DEFAULT 'member' CHECK (role IN ('member', 'admin')),
    joined_at BIGINT NOT NULL,
    CONSTRAINT uniq_membership UNIQUE (user_id, group_id),
    CONSTRAINT fk_membership_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_membership_group FOREIGN KEY (group_id) REFERENCES `groups`(id) ON DELETE CASCADE
);

CREATE INDEX idx_memberships_group_id ON memberships(group_id);
