SELECT t1.oid, t1.typname as basetype, t2.typname as arraytype,
       t2.typsubscript
FROM   pg_type t1 LEFT JOIN pg_type t2 ON (t1.typarray = t2.oid)
WHERE  t1.typarray <> 0 AND
       (t2.oid IS NULL OR
        t2.typsubscript <> 'array_subscript_handler'::regproc);
