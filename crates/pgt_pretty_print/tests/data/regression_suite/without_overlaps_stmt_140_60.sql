UPDATE  temporal_mltrng
SET     valid_at = '{[2020-01-01,2021-01-01)}'
WHERE   id = '[11,12)'
AND     valid_at @> '2018-01-15'::date;
