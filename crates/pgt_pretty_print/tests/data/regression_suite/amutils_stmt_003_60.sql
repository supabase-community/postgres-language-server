select prop,
       pg_index_has_property('onek_hundred'::regclass, prop) as btree,
       pg_index_has_property('hash_i4_index'::regclass, prop) as hash,
       pg_index_has_property('gcircleind'::regclass, prop) as gist,
       pg_index_has_property('sp_radix_ind'::regclass, prop) as spgist,
       pg_index_has_property('botharrayidx'::regclass, prop) as gin,
       pg_index_has_property('brinidx'::regclass, prop) as brin
  from unnest(array['clusterable', 'index_scan', 'bitmap_scan',
                    'backward_scan',
                    'bogus']::text[])
         with ordinality as u(prop,ord)
 order by ord;
