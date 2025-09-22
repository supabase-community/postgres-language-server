SELECT o1.oid, o1.oprname
FROM pg_operator AS o1
WHERE o1.oprcanhash AND NOT EXISTS
  (SELECT 1 FROM pg_amop
   WHERE amopmethod = (SELECT oid FROM pg_am WHERE amname = 'hash') AND
         amopopr = o1.oid AND amopstrategy = 1);
