SELECT indexrelid::regclass, indrelid::regclass, attname, atttypid::regtype, opcname
FROM (SELECT indexrelid, indrelid, unnest(indkey) as ikey,
             unnest(indclass) as iclass, unnest(indcollation) as icoll
      FROM pg_index
      WHERE indrelid < 16384) ss,
      pg_attribute a,
      pg_opclass opc
WHERE a.attrelid = indrelid AND a.attnum = ikey AND opc.oid = iclass AND
      (opcintype != atttypid OR icoll != attcollation)
ORDER BY 1;
