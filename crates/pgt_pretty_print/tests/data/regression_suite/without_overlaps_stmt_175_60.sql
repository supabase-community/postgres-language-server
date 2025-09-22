UPDATE  temporal_mltrng3
SET     id = NULL,
        valid_at = datemultirange(daterange('2020-01-01', '2021-01-01'))
WHERE   id = '[21,22)';
