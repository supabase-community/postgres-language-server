SELECT indexrelid::regclass, indrelid::regclass, iclass, icoll
FROM (SELECT indexrelid, indrelid,
             unnest(indclass) as iclass, unnest(indcollation) as icoll
      FROM pg_index
      WHERE indrelid < 16384) ss
WHERE icoll != 0 AND
    icoll != (SELECT oid FROM pg_collation WHERE collname = 'C');
