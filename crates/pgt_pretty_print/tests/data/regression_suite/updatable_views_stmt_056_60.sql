INSERT INTO rw_view15 (a) VALUES (3) ON CONFLICT (a) DO UPDATE set a = excluded.a;
