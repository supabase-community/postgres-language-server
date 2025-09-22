select count(*) from test_multirange_gist where mr @> int4multirange(int4range(10,20), int4range(30,40));
