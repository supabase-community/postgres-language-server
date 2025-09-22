SELECT indexrelid::regclass, indrelid::regclass, attname, atttypid::regtype, opcname
FROM (SELECT indexrelid, indrelid, unnest(indkey) as ikey,
             unnest(indclass) as iclass, unnest(indcollation) as icoll
      FROM pg_index) ss,
      pg_attribute a,
      pg_opclass opc
WHERE a.attrelid = indrelid AND a.attnum = ikey AND opc.oid = iclass AND
      (NOT binary_coercible(atttypid, opcintype) OR icoll != attcollation);
