select col, prop, pg_index_column_has_property(o, col, prop)
  from (values ('foocover'::regclass)) v1(o),
       (values (1,'orderable'),(2,'asc'),(3,'desc'),
               (4,'nulls_first'),(5,'nulls_last'),
               (6,'distance_orderable'),(7,'returnable'),
               (8, 'bogus')) v2(idx,prop),
       generate_series(1,3) col
 order by col, idx;
