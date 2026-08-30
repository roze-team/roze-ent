CREATE OR REPLACE VIEW user_activity_view AS
SELECT
    users.id AS user_id,
    users.email,
    users.name,
    CAST(COUNT(DISTINCT pets.id) AS SIGNED) AS pet_count,
    CAST(COUNT(DISTINCT memberships.group_id) AS SIGNED) AS group_count
FROM users
LEFT JOIN pets ON pets.owner_id = users.id
LEFT JOIN memberships ON memberships.user_id = users.id
GROUP BY users.id, users.email, users.name;
