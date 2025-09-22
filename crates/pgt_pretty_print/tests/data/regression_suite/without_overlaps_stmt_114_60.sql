UPDATE  temporal_rng3
SET     id = NULL,
        valid_at = 'empty'
WHERE   id = '[1,2)' AND valid_at IS NULL;
