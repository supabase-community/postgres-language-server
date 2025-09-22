SELECT a.aggfnoid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggdeserialfn = p.oid AND
    (p.prorettype != 'internal'::regtype OR p.pronargs != 2 OR
     p.proargtypes[0] != 'bytea'::regtype OR
     p.proargtypes[1] != 'internal'::regtype OR
     NOT p.proisstrict);
