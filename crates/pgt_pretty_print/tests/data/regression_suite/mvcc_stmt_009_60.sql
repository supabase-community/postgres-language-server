SELECT 'clean_aborted_self_key_before' AS size_before, pg_relation_size('clean_aborted_self_key') size_after
WHERE 'clean_aborted_self_key_before' != pg_relation_size('clean_aborted_self_key');
