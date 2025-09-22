UPDATE  temporal_rng3
SET     id = '[11,12)'
WHERE   id = '[1,2)'
AND     valid_at @> '2018-01-15'::date;
