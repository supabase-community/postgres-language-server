UPDATE  temporal_mltrng3
SET     id = '[21,22)',
        valid_at = '{[2018-01-02,2018-02-03)}'
WHERE   id = '[11,12)'
AND     valid_at @> '2020-01-15'::date;
