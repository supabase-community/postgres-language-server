SELECT a.aggfnoid::oid, p.proname, ptr.oid, ptr.proname
FROM pg_aggregate AS a, pg_proc AS p, pg_proc AS ptr
WHERE a.aggfnoid = p.oid AND
    a.aggtransfn = ptr.oid AND
    (ptr.proretset
     OR NOT (ptr.pronargs =
             CASE WHEN a.aggkind = 'n' THEN p.pronargs + 1
             ELSE greatest(p.pronargs - a.aggnumdirectargs, 1) + 1 END)
     OR NOT binary_coercible(ptr.prorettype, a.aggtranstype)
     OR NOT binary_coercible(a.aggtranstype, ptr.proargtypes[0])
     OR (p.pronargs > 0 AND
         NOT binary_coercible(p.proargtypes[0], ptr.proargtypes[1]))
     OR (p.pronargs > 1 AND
         NOT binary_coercible(p.proargtypes[1], ptr.proargtypes[2]))
     OR (p.pronargs > 2 AND
         NOT binary_coercible(p.proargtypes[2], ptr.proargtypes[3]))
     OR (p.pronargs > 3 AND
         NOT binary_coercible(p.proargtypes[3], ptr.proargtypes[4]))
     -- we could carry the check further, but 4 args is enough for now
     OR (p.pronargs > 4)
    );
