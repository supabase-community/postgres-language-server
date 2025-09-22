select prop,
       pg_index_column_has_property('onek_hundred'::regclass, 1, prop) as btree,
       pg_index_column_has_property('hash_i4_index'::regclass, 1, prop) as hash,
       pg_index_column_has_property('gcircleind'::regclass, 1, prop) as gist,
       pg_index_column_has_property('sp_radix_ind'::regclass, 1, prop) as spgist_radix,
       pg_index_column_has_property('sp_quad_ind'::regclass, 1, prop) as spgist_quad,
       pg_index_column_has_property('botharrayidx'::regclass, 1, prop) as gin,
       pg_index_column_has_property('brinidx'::regclass, 1, prop) as brin
  from unnest(array['asc', 'desc', 'nulls_first', 'nulls_last',
                    'orderable', 'distance_orderable', 'returnable',
                    'search_array', 'search_nulls',
                    'bogus']::text[])
         with ordinality as u(prop,ord)
 order by ord;
