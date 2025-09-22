INSERT INTO temporal_rng (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[3,4)';
