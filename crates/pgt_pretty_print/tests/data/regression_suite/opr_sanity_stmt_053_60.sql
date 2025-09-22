SELECT o1.oid, o1.oprcode, o2.oid, o2.oprcode
FROM pg_operator AS o1, pg_operator AS o2
WHERE o1.oid != o2.oid AND
    o1.oprname = o2.oprname AND
    o1.oprkind = o2.oprkind AND
    o1.oprleft = o2.oprleft AND
    o1.oprright = o2.oprright;
