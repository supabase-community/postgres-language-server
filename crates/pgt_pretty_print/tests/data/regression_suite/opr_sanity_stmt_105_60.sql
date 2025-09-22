SELECT c1.oid
FROM pg_opclass AS c1
WHERE c1.opcmethod = 0 OR c1.opcnamespace = 0 OR c1.opcfamily = 0
    OR c1.opcintype = 0;
