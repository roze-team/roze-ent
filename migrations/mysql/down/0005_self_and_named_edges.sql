DROP TABLE IF EXISTS friendships;
ALTER TABLE users DROP FOREIGN KEY fk_users_manager;
DROP INDEX idx_users_manager_id ON users;
ALTER TABLE users DROP COLUMN manager_id;
