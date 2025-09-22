SELECT o1.oid, o1.oprcode, o2.oid, o2.oprcode
FROM pg_operator AS o1, pg_operator AS o2
WHERE o1.oprnegate = o2.oid AND
    (o1.oprkind != o2.oprkind OR
     o1.oprleft != o2.oprleft OR
     o1.oprright != o2.oprright OR
     o1.oprresult != 'bool'::regtype OR
     o2.oprresult != 'bool'::regtype OR
     o1.oid != o2.oprnegate OR
     o1.oid = o2.oid);
