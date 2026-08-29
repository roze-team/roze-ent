DROP TABLE IF EXISTS friendships;
DROP INDEX IF EXISTS idx_users_manager_id;
ALTER TABLE users DROP COLUMN manager_id;
