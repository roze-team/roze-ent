SELECT setval(pg_get_serial_sequence('public.users', 'id'), COALESCE((SELECT MAX(id) FROM public.users), 1), EXISTS(SELECT 1 FROM public.users));
SELECT setval(pg_get_serial_sequence('public.pets', 'id'), COALESCE((SELECT MAX(id) FROM public.pets), 1), EXISTS(SELECT 1 FROM public.pets));
SELECT setval(pg_get_serial_sequence('public.groups', 'id'), COALESCE((SELECT MAX(id) FROM public.groups), 1), EXISTS(SELECT 1 FROM public.groups));
SELECT setval(pg_get_serial_sequence('public.memberships', 'id'), COALESCE((SELECT MAX(id) FROM public.memberships), 1), EXISTS(SELECT 1 FROM public.memberships));
SELECT setval(pg_get_serial_sequence('public.friendships', 'id'), COALESCE((SELECT MAX(id) FROM public.friendships), 1), EXISTS(SELECT 1 FROM public.friendships));
SELECT setval(pg_get_serial_sequence('public.projects', 'id'), COALESCE((SELECT MAX(id) FROM public.projects), 1), EXISTS(SELECT 1 FROM public.projects));
SELECT setval(pg_get_serial_sequence('roze_ent.audit_events', 'id'), COALESCE((SELECT MAX(id) FROM roze_ent.audit_events), 1), EXISTS(SELECT 1 FROM roze_ent.audit_events));
