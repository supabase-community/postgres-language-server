SELECT pc.oid, pc.relname, pa.amname, pa.amtype
FROM pg_class as pc JOIN pg_am AS pa ON (pc.relam = pa.oid)
WHERE pc.relkind IN ('r', 't', 'm') and
    pa.amtype != 't';
