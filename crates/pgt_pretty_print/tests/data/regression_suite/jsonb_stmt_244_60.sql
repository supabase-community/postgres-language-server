SELECT jsonb_build_object(
       'a', jsonb_build_object('b',false,'c',99),
       'd', jsonb_build_object('e',array[9,8,7]::int[],
           'f', (select row_to_json(r) from ( select relkind, oid::regclass as name from pg_class where relname = 'pg_class') r)));
