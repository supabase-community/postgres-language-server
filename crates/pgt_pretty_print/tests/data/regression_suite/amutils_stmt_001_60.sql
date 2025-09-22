select prop,
       pg_indexam_has_property(a.oid, prop) as "AM",
       pg_index_has_property('gcircleind'::regclass, prop) as "Index",
       pg_index_column_has_property('gcircleind'::regclass, 1, prop) as "Column"
  from pg_am a,
       unnest(array['asc', 'desc', 'nulls_first', 'nulls_last',
                    'orderable', 'distance_orderable', 'returnable',
                    'search_array', 'search_nulls',
                    'clusterable', 'index_scan', 'bitmap_scan',
                    'backward_scan',
                    'can_order', 'can_unique', 'can_multi_col',
                    'can_exclude', 'can_include',
                    'bogus']::text[])
         with ordinality as u(prop,ord)
 where a.amname = 'gist'
 order by ord;
