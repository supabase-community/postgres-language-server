SELECT count(*) as trigger_parallel_vacuum_nindexes
FROM pg_class
WHERE oid in ('regular_sized_index'::regclass, 'typically_sized_index'::regclass) AND
  pg_relation_size(oid) >=
  pg_size_bytes(current_setting('min_parallel_index_scan_size'));
