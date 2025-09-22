select count(*) from test_range_gist where ir &< int4multirange(int4range(100,200), int4range(400,500));
