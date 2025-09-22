UPDATE  temporal_rng3
SET     valid_at = daterange('2018-03-01', '2018-05-05')
WHERE   id = '[1,2)' AND valid_at IS NULL;
