SELECT t1.oid, t1.typname, t1.typalign, t2.typname, t2.typalign
FROM pg_type as t1
     LEFT JOIN pg_range as r ON rngtypid = t1.oid
     LEFT JOIN pg_type as t2 ON rngsubtype = t2.oid
WHERE t1.typtype = 'r' AND
    (t1.typalign != (CASE WHEN t2.typalign = 'd' THEN 'd'::"char"
                          ELSE 'i'::"char" END)
     OR t2.oid IS NULL);
