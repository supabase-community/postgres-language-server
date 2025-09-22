INSERT INTO temporal_partitioned_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
  ('[1,2)', datemultirange(daterange('2000-01-01', '2000-02-15')), '[1,2)'),
  ('[1,2)', datemultirange(daterange('2001-01-01', '2002-01-01')), '[2,3)'),
  ('[2,3)', datemultirange(daterange('2000-01-01', '2000-02-15')), '[1,2)');
