UPDATE  temporal_rng3
SET     id = '[1,2)',
        valid_at = NULL
WHERE   id IS NULL AND valid_at @> '2020-06-01'::date;
