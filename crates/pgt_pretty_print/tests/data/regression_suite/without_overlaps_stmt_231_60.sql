INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT ON CONSTRAINT temporal_rng_pk DO UPDATE SET id = EXCLUDED.id + '[2,3)';
