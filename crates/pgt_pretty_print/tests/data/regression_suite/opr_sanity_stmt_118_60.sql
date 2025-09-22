SELECT a1.amopfamily, a1.amopstrategy, a1.amopopr
FROM pg_amop AS a1
WHERE NOT EXISTS(SELECT 1 FROM pg_opclass AS c1
                 WHERE c1.opcfamily = a1.amopfamily
                   AND binary_coercible(c1.opcintype, a1.amoplefttype));
