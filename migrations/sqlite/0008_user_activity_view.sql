CREATE VIEW user_activity_view AS
SELECT
    users.id AS user_id,
    users.email,
    users.name,
    COUNT(DISTINCT pets.id) AS pet_count,
    COUNT(DISTINCT memberships.group_id) AS group_count
FROM users
LEFT JOIN pets ON pets.owner_id = users.id
LEFT JOIN memberships ON memberships.user_id = users.id
GROUP BY users.id, users.email, users.name;
