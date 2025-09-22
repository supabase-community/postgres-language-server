SELECT v as value, hash_array_extended(v, 0)::bit(32) as extended0
FROM   (VALUES ('{101}'::varbit[])) x(v);
