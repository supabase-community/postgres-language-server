INSERT INTO temporal_mltrng (id, valid_at) VALUES ('[1,2)', datemultirange(daterange('2005-01-01', '2006-01-01'))) ON CONFLICT ON CONSTRAINT temporal_mltrng_pk DO NOTHING;
