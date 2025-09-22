UPDATE temporal_rng SET id = '[7,8)'
  WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');
