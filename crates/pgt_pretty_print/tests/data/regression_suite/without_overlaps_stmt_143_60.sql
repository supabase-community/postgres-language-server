UPDATE  temporal_mltrng
SET     id = '[1,2)',
        valid_at = datemultirange(daterange('2018-03-05', '2018-05-05'))
WHERE   id = '[21,22)';
