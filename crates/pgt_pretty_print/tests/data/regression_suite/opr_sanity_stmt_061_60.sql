SELECT o1.oid, o1.oprname, p.amopfamily
FROM pg_operator AS o1, pg_amop p
WHERE amopopr = o1.oid
  AND amopmethod = (SELECT oid FROM pg_am WHERE amname = 'btree')
  AND amopstrategy = 3
  AND NOT o1.oprcanmerge;
