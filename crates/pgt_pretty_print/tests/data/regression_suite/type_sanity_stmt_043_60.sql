SELECT t.oid, t.typname, t.typanalyze
FROM pg_type t
WHERE t.typbasetype = 0 AND
    (t.typanalyze = 'array_typanalyze'::regproc) !=
    (t.typsubscript = 'array_subscript_handler'::regproc)
ORDER BY 1;
