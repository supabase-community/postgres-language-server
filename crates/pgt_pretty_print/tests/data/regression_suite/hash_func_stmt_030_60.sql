SELECT v as value, hash_multirange(v)::bit(32) as standard,
	   hash_multirange_extended(v, 0)::bit(32) as extended0,
	   hash_multirange_extended(v, 1)::bit(32) as extended1
FROM   (VALUES ('{[10,20)}'::int4multirange), ('{[23, 43]}'::int4multirange),
         ('{[5675, 550273)}'::int4multirange),
		 ('{[550274, 1550274)}'::int4multirange),
		 ('{[1550275, 208112489)}'::int4multirange)) x(v)
WHERE  hash_multirange(v)::bit(32) != hash_multirange_extended(v, 0)::bit(32)
       OR hash_multirange(v)::bit(32) = hash_multirange_extended(v, 1)::bit(32);
