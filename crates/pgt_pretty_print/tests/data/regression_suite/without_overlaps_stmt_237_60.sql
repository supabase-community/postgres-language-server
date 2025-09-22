INSERT INTO temporal3 (id, valid_at) VALUES ('[1,2)', daterange('2005-01-01', '2006-01-01')) ON CONFLICT DO NOTHING;
