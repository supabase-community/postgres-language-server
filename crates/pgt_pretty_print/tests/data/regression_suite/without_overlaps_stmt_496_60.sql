UPDATE temporal_partitioned_rng SET valid_at = daterange('2016-02-01', '2016-03-01')
  WHERE id = '[5,6)' AND valid_at = daterange('2018-02-01', '2018-03-01');
