INSERT INTO s2 (SELECT x, public.fipshash(x::text) FROM generate_series(-6,6) x);
