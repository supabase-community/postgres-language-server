INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2010-01-01', '2020-01-01')) ON CONFLICT ON CONSTRAINT temporal3_uq DO UPDATE SET id = EXCLUDED.id + '[3,4)';
