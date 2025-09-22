select count(*) from test_range_spgist where ir -|- int4range(100,500);
