UPDATE temporal_rng SET valid_at = daterange('2016-01-01', '2016-02-01')
  WHERE id = '[5,6)' AND valid_at = daterange('2018-01-01', '2018-02-01');
