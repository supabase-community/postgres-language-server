INSERT INTO temporal_mltrng3 (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2010-01-01', '2020-01-01'))) ON CONFLICT DO NOTHING;
