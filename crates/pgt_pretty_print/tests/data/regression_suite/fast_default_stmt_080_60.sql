SELECT attnum, attname, atthasmissing, atthasdef, attmissingval
FROM pg_attribute
WHERE attnum > 0 AND attrelid = 't2'::regclass
ORDER BY attnum;
