UPDATE temporal_partitioned_fk_mltrng2mltrng SET valid_at = datemultirange(daterange('2000-01-01', '2000-02-13')) WHERE id = '[2,3)';
