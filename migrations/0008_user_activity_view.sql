CREATE VIEW user_activity_view AS
SELECT
    users.id AS user_id,
    users.email,
    users.name,
    COUNT(DISTINCT pets.id)::BIGINT AS pet_count,
    COUNT(DISTINCT memberships.group_id)::BIGINT AS group_count
FROM public.users AS users
LEFT JOIN public.pets AS pets ON pets.owner_id = users.id
LEFT JOIN public.memberships AS memberships ON memberships.user_id = users.id
GROUP BY users.id, users.email, users.name;
