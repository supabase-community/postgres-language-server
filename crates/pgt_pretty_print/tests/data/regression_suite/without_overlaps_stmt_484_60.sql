INSERT INTO temporal_partitioned_fk_rng2rng (id, valid_at, parent_id) VALUES
  ('[1,2)', daterange('2000-01-01', '2000-02-15'), '[1,2)'),
  ('[1,2)', daterange('2001-01-01', '2002-01-01'), '[2,3)'),
  ('[2,3)', daterange('2000-01-01', '2000-02-15'), '[1,2)');
