select count(*) from test_range_gist where ir <@ '{(10,30),(40,60),(70,90)}'::int4multirange;
