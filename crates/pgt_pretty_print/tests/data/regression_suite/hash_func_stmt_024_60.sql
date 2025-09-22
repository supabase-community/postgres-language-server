SELECT v as value, pg_lsn_hash(v)::bit(32) as standard,
       pg_lsn_hash_extended(v, 0)::bit(32) as extended0,
       pg_lsn_hash_extended(v, 1)::bit(32) as extended1
FROM   (VALUES (NULL::pg_lsn), ('16/B374D84'), ('30/B374D84'),
        ('255/B374D84'), ('25/B379D90'), ('900/F37FD90')) x(v)
WHERE  pg_lsn_hash(v)::bit(32) != pg_lsn_hash_extended(v, 0)::bit(32)
       OR pg_lsn_hash(v)::bit(32) = pg_lsn_hash_extended(v, 1)::bit(32);
