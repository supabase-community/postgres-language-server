UPDATE temporal_rng SET valid_at = daterange('2018-01-01', '2018-03-01')
  WHERE id = '[1,2)' AND valid_at @> '2018-01-25'::date;
