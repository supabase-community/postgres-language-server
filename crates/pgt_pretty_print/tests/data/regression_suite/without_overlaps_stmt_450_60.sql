UPDATE temporal_mltrng
  SET valid_at = CASE WHEN lower(valid_at) = '2018-01-01' THEN datemultirange(daterange('2018-01-01', '2018-01-05'))
                      WHEN lower(valid_at) = '2018-02-01' THEN datemultirange(daterange('2018-01-05', '2018-03-01')) END
  WHERE id = '[6,7)';
