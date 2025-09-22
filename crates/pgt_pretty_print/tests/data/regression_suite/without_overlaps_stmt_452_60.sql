INSERT INTO temporal_fk_mltrng2mltrng (id, valid_at, parent_id) VALUES
  ('[1,2)', datemultirange(daterange('2018-01-15', '2018-02-01')), '[1,2)'),
  ('[2,3)', datemultirange(daterange('2018-01-15', '2018-05-01')), '[1,2)');
