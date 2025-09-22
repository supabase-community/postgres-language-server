UPDATE  temporal_rng3
SET     id = NULL,
        valid_at = daterange('2020-01-01', '2021-01-01')
WHERE   id = '[21,22)';
