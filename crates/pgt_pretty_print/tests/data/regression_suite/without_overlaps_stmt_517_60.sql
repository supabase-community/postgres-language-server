UPDATE temporal_partitioned_fk_mltrng2mltrng SET valid_at = datemultirange(daterange('2000-01-01', '2000-04-01')) WHERE id = '[1,2)';
