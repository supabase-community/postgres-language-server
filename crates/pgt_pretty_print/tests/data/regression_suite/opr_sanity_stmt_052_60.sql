SELECT o1.oid, o1.oprname
FROM pg_operator as o1
WHERE (o1.oprleft = 0 and o1.oprkind != 'l') OR
    (o1.oprleft != 0 and o1.oprkind = 'l') OR
    o1.oprright = 0;
