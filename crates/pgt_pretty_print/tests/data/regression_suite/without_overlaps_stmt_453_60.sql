UPDATE temporal_mltrng SET valid_at = datemultirange(daterange('2018-01-15', '2018-03-01'))
  WHERE id = '[1,2)' AND valid_at @> '2018-01-15'::date;
