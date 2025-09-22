SELECT o1.oid, o1.oprname
FROM pg_operator as o1 LEFT JOIN pg_description as d
     ON o1.tableoid = d.classoid and o1.oid = d.objoid and d.objsubid = 0
WHERE d.classoid IS NULL AND o1.oid <= 9999;
