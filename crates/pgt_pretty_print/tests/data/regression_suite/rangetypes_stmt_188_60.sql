select count(*) from test_range_gist where ir && '{(10,20),(30,40),(50,60)}'::int4multirange;
