DELETE FROM sqlite_sequence WHERE name IN ('users', 'pets', 'groups', 'memberships', 'friendships', 'projects', 'audit_events');
INSERT INTO sqlite_sequence(name, seq) VALUES ('users', 0);
INSERT INTO sqlite_sequence(name, seq) VALUES ('pets', 4294967295);
INSERT INTO sqlite_sequence(name, seq) VALUES ('groups', 8589934591);
INSERT INTO sqlite_sequence(name, seq) VALUES ('memberships', 12884901887);
INSERT INTO sqlite_sequence(name, seq) VALUES ('friendships', 17179869183);
INSERT INTO sqlite_sequence(name, seq) VALUES ('projects', 21474836479);
INSERT INTO sqlite_sequence(name, seq) VALUES ('audit_events', 25769803775);
