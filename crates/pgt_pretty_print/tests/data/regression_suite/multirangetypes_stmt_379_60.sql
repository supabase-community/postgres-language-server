select count(*) from test_multirange_gist where mr <@ 'empty'::int4range;
