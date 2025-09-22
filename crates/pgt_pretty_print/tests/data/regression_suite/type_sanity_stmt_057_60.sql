SELECT r.rngtypid, r.rngsubtype, o.opcmethod, o.opcname
FROM pg_range r JOIN pg_opclass o ON o.oid = r.rngsubopc
WHERE o.opcmethod != 403 OR
    ((o.opcintype != r.rngsubtype) AND NOT
     (o.opcintype = 'pg_catalog.anyarray'::regtype AND
      EXISTS(select 1 from pg_catalog.pg_type where
             oid = r.rngsubtype and typelem != 0 and
             typsubscript = 'array_subscript_handler'::regproc)));
