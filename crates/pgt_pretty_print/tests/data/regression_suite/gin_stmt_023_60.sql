insert into t_gin_test_tbl
values
  (null,    null),
  ('{}',    null),
  ('{1}',   null),
  ('{1,2}', null),
  (null,    '{}'),
  (null,    '{10}'),
  ('{1,2}', '{10}'),
  ('{2}',   '{10}'),
  ('{1,3}', '{}'),
  ('{1,1}', '{10}');
