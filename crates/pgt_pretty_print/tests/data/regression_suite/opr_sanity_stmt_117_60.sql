SELECT c1.opcname, c1.opcfamily
FROM pg_opclass AS c1
WHERE NOT EXISTS(SELECT 1 FROM pg_amop AS a1
                 WHERE a1.amopfamily = c1.opcfamily
                   AND binary_coercible(c1.opcintype, a1.amoplefttype));
