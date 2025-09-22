ALTER TABLE temporal_fk_rng2rng
  ALTER COLUMN valid_at TYPE daterange USING daterange(lower(valid_at)::date, upper(valid_at)::date);
