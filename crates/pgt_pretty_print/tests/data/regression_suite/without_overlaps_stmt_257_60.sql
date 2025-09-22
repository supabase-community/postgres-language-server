INSERT INTO temporal3 (id, valid_at) VALUES ('[2,3)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[4,5)';
