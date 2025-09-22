select count(*) from test_range_elem where int4range(i,i+10) <@ int4range(10,30);
