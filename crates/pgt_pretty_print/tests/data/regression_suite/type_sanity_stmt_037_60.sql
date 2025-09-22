SELECT t1.oid, t1.typname, t1.typelem
FROM pg_type AS t1
WHERE t1.typelem != 0 AND t1.typsubscript = 0;
