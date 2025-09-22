SELECT v as value, hash_array(v)::bit(32) as standard
FROM   (VALUES ('{101}'::varbit[])) x(v);
