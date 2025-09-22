INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT ON CONSTRAINT temporal_rng_pk DO NOTHING;
