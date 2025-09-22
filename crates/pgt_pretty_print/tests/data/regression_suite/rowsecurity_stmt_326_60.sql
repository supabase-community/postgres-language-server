INSERT INTO b1 (SELECT x, public.fipshash(x::text) FROM generate_series(-10,10) x);
