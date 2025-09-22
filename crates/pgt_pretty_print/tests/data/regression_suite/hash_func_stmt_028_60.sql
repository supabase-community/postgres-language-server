SELECT v as value, jsonb_hash(v)::bit(32) as standard,
       jsonb_hash_extended(v, 0)::bit(32) as extended0,
       jsonb_hash_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::jsonb),
        ('{"a": "aaa bbb ddd ccc", "b": ["eee fff ggg"], "c": {"d": "hhh iii"}}'),
        ('{"foo": [true, "bar"], "tags": {"e": 1, "f": null}}'),
        ('{"g": {"h": "value"}}')) x(v)
WHERE  jsonb_hash(v)::bit(32) != jsonb_hash_extended(v, 0)::bit(32)
       OR jsonb_hash(v)::bit(32) = jsonb_hash_extended(v, 1)::bit(32);
