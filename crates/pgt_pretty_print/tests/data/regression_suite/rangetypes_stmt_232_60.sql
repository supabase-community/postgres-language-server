select count(*) from test_range_gist where ir @> int4multirange(int4range(10,20), int4range(30,40));
