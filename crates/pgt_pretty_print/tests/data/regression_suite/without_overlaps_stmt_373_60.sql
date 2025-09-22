INSERT INTO temporal_fk_rng2rng (id, valid_at, parent_id) VALUES
  ('[1,2)', daterange('2018-01-15', '2018-02-01'), '[1,2)'),
  ('[2,3)', daterange('2018-01-15', '2018-05-01'), '[1,2)');
