UPDATE  temporal_rng
SET     id = NULL,
        valid_at = daterange('2018-03-05', '2018-05-05')
WHERE   id = '[21,22)';
