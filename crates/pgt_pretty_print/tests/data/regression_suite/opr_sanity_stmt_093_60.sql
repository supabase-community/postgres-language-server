SELECT a.aggfnoid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggserialfn = p.oid AND
    (p.prorettype != 'bytea'::regtype OR p.pronargs != 1 OR
     p.proargtypes[0] != 'internal'::regtype OR
     NOT p.proisstrict);
