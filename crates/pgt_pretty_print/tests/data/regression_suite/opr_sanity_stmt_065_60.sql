SELECT o1.oid, o1.oprname, p1.oid, p1.proname
FROM pg_operator AS o1, pg_proc AS p1
WHERE o1.oprcode = p1.oid AND
    o1.oprkind = 'l' AND
    (p1.pronargs != 1
     OR NOT binary_coercible(p1.prorettype, o1.oprresult)
     OR NOT binary_coercible(o1.oprright, p1.proargtypes[0])
     OR o1.oprleft != 0);
