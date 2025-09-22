insert into test_range_spgist select 'empty'::int4range from generate_series(1,500) g;
