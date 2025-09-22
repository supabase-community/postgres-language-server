ALTER TABLE temporal_fk_rng2rng
  DROP CONSTRAINT temporal_fk_rng2rng_fk,
  ALTER COLUMN valid_at TYPE tsrange USING tsrange(lower(valid_at), upper(valid_at));
