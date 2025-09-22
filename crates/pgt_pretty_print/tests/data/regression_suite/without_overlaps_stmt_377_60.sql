UPDATE temporal_rng SET id = '[2,3)'
  WHERE id = '[1,2)' AND valid_at @> '2018-01-15'::date;
