SELECT v as value, uuid_hash(v)::bit(32) as standard,
       uuid_hash_extended(v, 0)::bit(32) as extended0,
       uuid_hash_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::uuid), ('a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'),
        ('5a9ba4ac-8d6f-11e7-bb31-be2e44b06b34'),
        ('99c6705c-d939-461c-a3c9-1690ad64ed7b'),
        ('7deed3ca-8d6f-11e7-bb31-be2e44b06b34'),
        ('9ad46d4f-6f2a-4edd-aadb-745993928e1e')) x(v)
WHERE  uuid_hash(v)::bit(32) != uuid_hash_extended(v, 0)::bit(32)
       OR uuid_hash(v)::bit(32) = uuid_hash_extended(v, 1)::bit(32);
