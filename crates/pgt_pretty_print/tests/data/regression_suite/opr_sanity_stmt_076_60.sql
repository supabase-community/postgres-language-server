SELECT a.aggfnoid::oid, p.proname
FROM pg_aggregate as a, pg_proc as p
WHERE a.aggfnoid = p.oid AND
    (p.prokind != 'a' OR p.proretset OR p.pronargs < a.aggnumdirectargs);
