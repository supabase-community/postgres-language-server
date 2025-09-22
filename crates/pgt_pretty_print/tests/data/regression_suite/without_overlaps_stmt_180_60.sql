UPDATE  temporal_mltrng3
SET     id = NULL,
        valid_at = '{}'
WHERE   id = '[1,2)' AND valid_at IS NULL;
