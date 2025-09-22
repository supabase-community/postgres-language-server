SELECT a1.oid, f1.oid
FROM pg_amop AS a1, pg_opfamily AS f1
WHERE a1.amopfamily = f1.oid AND a1.amopmethod != f1.opfmethod;
