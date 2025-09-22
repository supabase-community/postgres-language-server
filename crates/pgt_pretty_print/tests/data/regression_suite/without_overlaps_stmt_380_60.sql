UPDATE temporal_rng SET valid_at = daterange('2018-01-15', '2018-02-15')
  WHERE id = '[2,3)';
