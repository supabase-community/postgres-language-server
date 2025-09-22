UPDATE temporal_partitioned_mltrng SET valid_at = datemultirange(daterange('2018-01-01', '2018-02-01')) WHERE id = '[5,6)';
