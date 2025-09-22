select count(*) from test_range_gist where ir @> 'empty'::int4range;
