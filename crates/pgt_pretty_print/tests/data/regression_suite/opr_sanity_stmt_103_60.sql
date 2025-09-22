SELECT f.oid
FROM pg_opfamily as f
WHERE f.opfmethod = 0 OR f.opfnamespace = 0;
