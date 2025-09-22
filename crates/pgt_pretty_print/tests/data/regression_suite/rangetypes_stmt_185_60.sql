select count(*) from test_range_gist where ir -|- int4range(100,500);
