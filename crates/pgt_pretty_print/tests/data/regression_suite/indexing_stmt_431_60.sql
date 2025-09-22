insert into idxpart select 2^g, format('two to power of %s', g) from generate_series(15, 17) g;
