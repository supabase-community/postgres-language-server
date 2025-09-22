INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT (id, valid_at) DO UPDATE SET id = EXCLUDED.id + '[2,3)';
