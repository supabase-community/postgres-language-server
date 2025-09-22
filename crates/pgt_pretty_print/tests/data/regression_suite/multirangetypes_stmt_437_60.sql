select count(*) from test_multirange_gist where mr @> int4range(10,20);
