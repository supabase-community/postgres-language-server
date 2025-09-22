UPDATE temporal_mltrng SET valid_at = datemultirange(daterange('2018-01-15', '2018-02-15'))
  WHERE id = '[2,3)';
