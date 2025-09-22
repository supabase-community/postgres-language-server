select count(*) from test_multirange_gist where mr && '{(10,20),(30,40),(50,60)}'::int4multirange;
