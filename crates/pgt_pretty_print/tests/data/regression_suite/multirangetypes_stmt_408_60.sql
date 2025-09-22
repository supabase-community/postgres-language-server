select count(*) from test_multirange_gist where mr << int4multirange(int4range(100,200), int4range(400,500));
