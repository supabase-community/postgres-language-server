SELECT o1.oid, o1.oprcode, o2.oid, o2.oprcode
FROM pg_operator AS o1, pg_operator AS o2
WHERE o1.oprcom = o2.oid AND
    (o1.oprkind != 'b' OR
     o1.oprleft != o2.oprright OR
     o1.oprright != o2.oprleft OR
     o1.oprresult != o2.oprresult OR
     o1.oid != o2.oprcom);
