SELECT c1.oid, f1.oid
FROM pg_opclass AS c1, pg_opfamily AS f1
WHERE c1.opcfamily = f1.oid AND c1.opcmethod != f1.opfmethod;
