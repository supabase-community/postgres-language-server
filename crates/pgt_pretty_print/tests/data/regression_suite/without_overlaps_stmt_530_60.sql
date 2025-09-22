DELETE FROM temporal_partitioned_mltrng WHERE id = '[5,6)' AND valid_at = datemultirange(daterange('2018-01-01', '2018-02-01'));
