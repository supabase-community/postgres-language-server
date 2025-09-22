UPDATE temporal_partitioned_mltrng SET valid_at = datemultirange(daterange('2016-02-01', '2016-03-01'))
  WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-02-01', '2018-03-01'));
