INSERT INTO temporal_partitioned_mltrng (id, valid_at, name) VALUES
  ('[1,2)', datemultirange(daterange('2000-01-01', '2000-02-01')), 'one'),
  ('[1,2)', datemultirange(daterange('2000-02-01', '2000-03-01')), 'one'),
  ('[2,3)', datemultirange(daterange('2000-01-01', '2010-01-01')), 'two');
